#!/usr/bin/env python3
"""Reasoning-content shim proxy.

Some thinking-mode LLMs (Kimi-k2.6, DeepSeek-v4-pro, etc.) reject multi-turn
tool-call flows when the prior assistant message does not carry a
reasoning_content field. The substrate we run (ironclaw / hermes / openclaw)
does not round-trip reasoning into chat history.

This proxy sits between the substrate and the upstream OpenAI-compatible
endpoint. It walks the messages array and, for any assistant message that has
non-empty tool_calls and is missing (or has empty) reasoning_content, injects
a placeholder. Empirically all known thinking-mode validators accept any
non-empty string here.

Usage:
    UPSTREAM_BASE_URL=https://api.deepseek.com \
    UPSTREAM_API_KEY=sk-... \
    PROXY_PORT=8000 \
    .venv/bin/python reasoning_shim_proxy.py

Substrate then talks to http://localhost:8000/v1 instead of the upstream URL.
"""

import hashlib
import json
import os
import re
import sys
import threading
import time

import aiohttp
from aiohttp import web

UPSTREAM_BASE_URL = os.environ.get("UPSTREAM_BASE_URL", "").rstrip("/")
UPSTREAM_API_KEY = os.environ.get("UPSTREAM_API_KEY")
PROXY_PORT = int(os.environ.get("PROXY_PORT", "8000"))
PLACEHOLDER = os.environ.get("REASONING_PLACEHOLDER", "(omitted)")
CAPTURE_PATH = os.environ.get("REASONING_CAPTURE_PATH", "/tmp/shim_reasoning.jsonl")
_CAPTURE_LOCK = threading.Lock()

# Substrates (e.g. openclaw) parse `provider/model` on the first slash and
# disallow further slashes in the model id. Map sanitized id -> upstream id.
MODEL_ALIASES = {
    "qwen-3-5-122b-a10b": "qwen/qwen3.5-122b-a10b",
}

if not UPSTREAM_BASE_URL:
    print("UPSTREAM_BASE_URL must be set", file=sys.stderr)
    sys.exit(1)


THINKING_INLINE_RE = re.compile(r"\[thinking\].*?\[/thinking\]\s*", re.DOTALL)


def strip_inline_thinking(text):
    if not isinstance(text, str) or "[thinking]" not in text:
        return text
    return THINKING_INLINE_RE.sub("", text)


def patch_messages(messages):
    """Inject placeholder reasoning_content for tool-call turns; strip
    inlined [thinking]...[/thinking] tags from assistant content so the
    upstream LLM doesn't see them as part of the prior assistant turn."""
    patched = 0
    stripped = 0
    for msg in messages:
        if msg.get("role") != "assistant":
            continue
        content = msg.get("content")
        if isinstance(content, str) and "[thinking]" in content:
            new_content = strip_inline_thinking(content)
            if new_content != content:
                msg["content"] = new_content if new_content else None
                stripped += 1
        if msg.get("tool_calls"):
            rc = msg.get("reasoning_content")
            if not rc:
                msg["reasoning_content"] = PLACEHOLDER
                patched += 1
    return patched, stripped


def _normalize_reasoning_field(msg):
    """Some upstreams (OpenRouter for Qwen) return reasoning on `reasoning`
    instead of `reasoning_content`. Mirror it so substrates that key on
    `reasoning_content` find it."""
    rc = msg.get("reasoning_content")
    if isinstance(rc, str) and rc.strip():
        return
    alt = msg.get("reasoning")
    if isinstance(alt, str) and alt.strip():
        msg["reasoning_content"] = alt

def _last_user_text_hash(payload):
    """Hash of the last user message in the request. Substrate runners
    can correlate captured responses by replaying their own prompt's hash."""
    if not isinstance(payload, dict):
        return None
    msgs = payload.get("messages") or []
    for m in reversed(msgs):
        if isinstance(m, dict) and m.get("role") == "user":
            c = m.get("content") or ""
            if isinstance(c, list):
                c = "".join(p.get("text", "") for p in c if isinstance(p, dict))
            return hashlib.sha256((c or "").encode("utf-8")).hexdigest()[:16]
    return None


def capture_response(payload, resp_json):
    """Append a JSONL record describing this LLM response. Substrate
    runners read this file (filtered by time-window or hash) to populate
    blue_reasoning without per-substrate reasoning extraction."""
    try:
        if not isinstance(resp_json, dict):
            return
        choices = resp_json.get("choices") or []
        msg = (choices[0].get("message") if choices and isinstance(choices[0], dict) else None) or {}
        rc = msg.get("reasoning_content") or msg.get("reasoning") or ""
        content = msg.get("content") or ""
        record = {
            "ts": time.time(),
            "ts_iso": time.strftime("%Y-%m-%dT%H:%M:%S", time.gmtime()) + "Z",
            "id": resp_json.get("id"),
            "model": resp_json.get("model") or (payload or {}).get("model"),
            "user_hash": _last_user_text_hash(payload),
            "reasoning_content": rc if isinstance(rc, str) else "",
            "reasoning_len": len(rc) if isinstance(rc, str) else 0,
            "content_len": len(content) if isinstance(content, str) else 0,
            "content_head": (content[:600] if isinstance(content, str) else ""),
        }
        line = json.dumps(record, ensure_ascii=False) + "\n"
        with _CAPTURE_LOCK:
            with open(CAPTURE_PATH, "a", encoding="utf-8") as f:
                f.write(line)
    except Exception as e:
        print(f"[proxy] capture_response error: {e}", file=sys.stderr, flush=True)


def inline_thinking_in_choices(resp_json):
    """For each choice, prepend [thinking]reasoning_content[/thinking] to
    message.content so substrates that don't read reasoning_content
    top-level still surface thinking in their text-content stream."""
    if not isinstance(resp_json, dict):
        return 0
    choices = resp_json.get("choices")
    if not isinstance(choices, list):
        return 0
    inlined = 0
    for choice in choices:
        msg = choice.get("message") if isinstance(choice, dict) else None
        if not isinstance(msg, dict):
            continue
        _normalize_reasoning_field(msg)
        rc = msg.get("reasoning_content")
        if not isinstance(rc, str) or not rc.strip() or rc == PLACEHOLDER:
            continue
        text = msg.get("content") or ""
        if not isinstance(text, str):
            continue
        if "[thinking]" in text:
            continue
        inline = "[thinking]" + rc + "[/thinking]"
        msg["content"] = inline + ("\n\n" + text if text else "")
        inlined += 1
    return inlined


async def handle(request: web.Request) -> web.StreamResponse:
    body = await request.read()
    try:
        payload = json.loads(body)
    except json.JSONDecodeError:
        payload = None

    patched = 0
    stripped = 0
    is_stream_request = False
    aliased = False
    if isinstance(payload, dict):
        if isinstance(payload.get("messages"), list):
            patched, stripped = patch_messages(payload["messages"])
            is_stream_request = bool(payload.get("stream"))
        m = payload.get("model")
        if isinstance(m, str) and m in MODEL_ALIASES:
            payload["model"] = MODEL_ALIASES[m]
            aliased = True
        body = json.dumps(payload).encode("utf-8")

    headers = {k: v for k, v in request.headers.items()
               if k.lower() not in ("host", "content-length", "authorization")}
    headers["Authorization"] = f"Bearer {UPSTREAM_API_KEY}" if UPSTREAM_API_KEY else request.headers.get("Authorization", "")
    headers["Content-Length"] = str(len(body))

    upstream_url = UPSTREAM_BASE_URL + request.path_qs

    print(f"[proxy] {request.method} {request.path_qs} -> {upstream_url} "
          f"(patched={patched} stripped={stripped} stream={is_stream_request})",
          file=sys.stderr, flush=True)

    timeout = aiohttp.ClientTimeout(total=600)
    async with aiohttp.ClientSession(timeout=timeout) as session:
        async with session.request(
            request.method, upstream_url, data=body, headers=headers,
        ) as upstream:
            up_ct = upstream.headers.get("content-type", "")
            buffer_response = (
                upstream.status == 200
                and not is_stream_request
                and "application/json" in up_ct
            )
            if buffer_response:
                resp_body = await upstream.read()
                inlined = 0
                try:
                    resp_json = json.loads(resp_body)
                    capture_response(payload, resp_json)
                    inlined = inline_thinking_in_choices(resp_json)
                    if inlined:
                        resp_body = json.dumps(resp_json).encode("utf-8")
                except json.JSONDecodeError:
                    pass
                if inlined:
                    print(f"[proxy] inlined thinking on {inlined} choices",
                          file=sys.stderr, flush=True)
                resp_headers = {k: v for k, v in upstream.headers.items()
                                if k.lower() not in ("content-length", "transfer-encoding")}
                return web.Response(body=resp_body, status=upstream.status, headers=resp_headers)

            resp = web.StreamResponse(status=upstream.status,
                                      headers={k: v for k, v in upstream.headers.items()
                                               if k.lower() not in ("content-length", "transfer-encoding")})
            await resp.prepare(request)
            async for chunk in upstream.content.iter_any():
                await resp.write(chunk)
            await resp.write_eof()
            return resp


def main():
    app = web.Application(client_max_size=1024 * 1024 * 32)
    app.router.add_route("*", "/{path:.*}", handle)
    print(f"[proxy] listening on 0.0.0.0:{PROXY_PORT} -> {UPSTREAM_BASE_URL}", file=sys.stderr)
    web.run_app(app, host="0.0.0.0", port=PROXY_PORT, print=None)


if __name__ == "__main__":
    main()
