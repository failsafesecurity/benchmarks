#!/usr/bin/env python3
"""OpenAI-compat <-> Anthropic /v1/messages translator proxy.

Lets ironclaw/hermes/openclaw talk to claude-sonnet-4-6 with extended thinking,
exposing thinking blocks as OpenAI-shape `reasoning_content` so the substrate's
existing reasoning capture surfaces them.

Round-trips thinking signatures through reasoning_content (encoded as a
trailing [signature]...[/signature] tag) so multi-turn tool flows preserve the
cached-thinking signature Anthropic requires.

Usage:
    UPSTREAM_API_KEY=sk-ant-... PROXY_PORT=8767 \\
    THINKING_BUDGET_TOKENS=4096 \\
    .venv/bin/python anthropic_translator_proxy.py
"""
import json
import os
import re
import sys

import aiohttp
from aiohttp import web

UPSTREAM_API_KEY = os.environ.get("UPSTREAM_API_KEY")
PROXY_PORT = int(os.environ.get("PROXY_PORT", "8767"))
THINKING_BUDGET = int(os.environ.get("THINKING_BUDGET_TOKENS", "4096"))
ANTHROPIC_URL = "https://api.anthropic.com/v1/messages"
ANTHROPIC_VERSION = "2023-06-01"

if not UPSTREAM_API_KEY:
    print("UPSTREAM_API_KEY must be set", file=sys.stderr)
    sys.exit(1)


def encode_reasoning(text: str, signature):
    if not signature:
        return text
    return (text or "") + "\n\n[signature]" + signature + "[/signature]"


def decode_reasoning(rc: str):
    m = re.search(r"\[signature\](.+?)\[/signature\]\s*$", rc, re.DOTALL)
    if not m:
        return rc, None
    return rc[: m.start()].rstrip("\n"), m.group(1)


THINKING_INLINE_RE = re.compile(r"\[thinking\].*?\[/thinking\]\s*", re.DOTALL)


def strip_inline_thinking(text: str) -> str:
    if not text or "[thinking]" not in text:
        return text
    return THINKING_INLINE_RE.sub("", text)


def openai_tools_to_anthropic(tools):
    out = []
    for t in tools or []:
        if t.get("type") == "function":
            f = t.get("function", {})
            out.append({
                "name": f.get("name"),
                "description": f.get("description", ""),
                "input_schema": f.get("parameters", {"type": "object"}),
            })
    return out


def messages_oai_to_anthropic(messages):
    system = None
    out = []
    pending_tool_results = []

    def flush_tool_results():
        nonlocal pending_tool_results
        if pending_tool_results:
            out.append({"role": "user", "content": pending_tool_results})
            pending_tool_results = []

    for m in messages:
        role = m.get("role")
        if role == "system":
            sc = m.get("content")
            if isinstance(sc, list):
                sc = "\n".join(b.get("text", "") for b in sc if b.get("type") == "text")
            system = sc
            continue

        if role == "tool":
            tc_id = m.get("tool_call_id")
            tc_content = m.get("content", "")
            if isinstance(tc_content, list):
                tc_content = json.dumps(tc_content)
            elif not isinstance(tc_content, str):
                tc_content = json.dumps(tc_content)
            pending_tool_results.append({
                "type": "tool_result",
                "tool_use_id": tc_id,
                "content": tc_content,
            })
            continue

        flush_tool_results()

        if role == "user":
            content = m.get("content")
            if isinstance(content, str):
                out.append({"role": "user", "content": content})
            elif isinstance(content, list):
                blocks = []
                for b in content:
                    if isinstance(b, str):
                        blocks.append({"type": "text", "text": b})
                    elif isinstance(b, dict) and b.get("type") == "text":
                        blocks.append({"type": "text", "text": b.get("text", "")})
                out.append({"role": "user", "content": blocks or [{"type": "text", "text": ""}]})
            else:
                out.append({"role": "user", "content": ""})

        elif role == "assistant":
            blocks = []
            rc = m.get("reasoning_content")
            tool_calls = m.get("tool_calls") or []
            if rc and tool_calls:
                text, sig = decode_reasoning(rc)
                if sig:
                    blocks.append({"type": "thinking", "thinking": text, "signature": sig})

            content = m.get("content")
            if isinstance(content, str) and content:
                stripped = strip_inline_thinking(content)
                if stripped:
                    blocks.append({"type": "text", "text": stripped})
            elif isinstance(content, list):
                for b in content:
                    if isinstance(b, dict) and b.get("type") == "text":
                        stripped = strip_inline_thinking(b.get("text", ""))
                        if stripped:
                            blocks.append({"type": "text", "text": stripped})

            for tc in tool_calls:
                f = tc.get("function", {})
                args = f.get("arguments")
                if isinstance(args, str):
                    try:
                        args = json.loads(args) if args else {}
                    except json.JSONDecodeError:
                        args = {"_raw": args}
                blocks.append({
                    "type": "tool_use",
                    "id": tc.get("id"),
                    "name": f.get("name"),
                    "input": args or {},
                })

            if blocks:
                out.append({"role": "assistant", "content": blocks})

    flush_tool_results()
    return system, out


def _sse_event(payload: dict) -> bytes:
    return b"data: " + json.dumps(payload).encode("utf-8") + b"\n\n"


def openai_response_to_sse_chunks(oai: dict) -> bytes:
    """Convert a buffered OpenAI chat-completion response into a series of SSE
    chunk events. Hermes (and other openai-sdk clients) call with stream=True
    and silently drop a non-SSE response, so we materialize a fake stream from
    the buffered upstream JSON.
    """
    cid = oai.get("id", "msg_unknown")
    model = oai.get("model", "")
    created = oai.get("created", 0)
    choice = (oai.get("choices") or [{}])[0]
    msg = choice.get("message") or {}
    finish_reason = choice.get("finish_reason", "stop")

    base = {
        "id": cid,
        "object": "chat.completion.chunk",
        "created": created,
        "model": model,
    }
    out = bytearray()

    # role chunk
    out += _sse_event({**base, "choices": [{"index": 0, "delta": {"role": "assistant"}, "finish_reason": None}]})

    # reasoning_content (some clients read this from delta)
    rc = msg.get("reasoning_content")
    if rc:
        out += _sse_event({**base, "choices": [{"index": 0, "delta": {"reasoning_content": rc}, "finish_reason": None}]})

    # content
    content = msg.get("content")
    if content:
        out += _sse_event({**base, "choices": [{"index": 0, "delta": {"content": content}, "finish_reason": None}]})

    # tool_calls — emit each as a single delta with full id/name/arguments
    tool_calls = msg.get("tool_calls") or []
    for i, tc in enumerate(tool_calls):
        delta_tc = {
            "index": i,
            "id": tc.get("id"),
            "type": tc.get("type", "function"),
            "function": {
                "name": (tc.get("function") or {}).get("name", ""),
                "arguments": (tc.get("function") or {}).get("arguments", ""),
            },
        }
        out += _sse_event({**base, "choices": [{"index": 0, "delta": {"tool_calls": [delta_tc]}, "finish_reason": None}]})

    # finish chunk
    out += _sse_event({**base, "choices": [{"index": 0, "delta": {}, "finish_reason": finish_reason}]})

    # usage chunk
    usage = oai.get("usage")
    if usage:
        out += _sse_event({**base, "choices": [], "usage": usage})

    out += b"data: [DONE]\n\n"
    return bytes(out)


def anthropic_response_to_openai(resp, requested_model):
    content_blocks = resp.get("content", [])

    text_parts = []
    thinking_text = ""
    thinking_signature = None
    tool_calls = []

    for block in content_blocks:
        btype = block.get("type")
        if btype == "thinking":
            thinking_text = block.get("thinking", "")
            thinking_signature = block.get("signature")
        elif btype == "text":
            text_parts.append(block.get("text", ""))
        elif btype == "tool_use":
            tool_calls.append({
                "id": block.get("id"),
                "type": "function",
                "function": {
                    "name": block.get("name"),
                    "arguments": json.dumps(block.get("input", {})),
                },
            })

    text_combined = "".join(text_parts)
    if thinking_text:
        inline = "[thinking]" + thinking_text + "[/thinking]"
        text_combined = inline + ("\n\n" + text_combined if text_combined else "")
    message = {"role": "assistant", "content": text_combined if text_combined else None}
    if thinking_text or thinking_signature:
        message["reasoning_content"] = encode_reasoning(thinking_text, thinking_signature)
    if tool_calls:
        message["tool_calls"] = tool_calls

    stop_reason = resp.get("stop_reason")
    finish_reason = {
        "end_turn": "stop",
        "max_tokens": "length",
        "stop_sequence": "stop",
        "tool_use": "tool_calls",
    }.get(stop_reason, "stop")

    usage = resp.get("usage", {})
    return {
        "id": resp.get("id", "msg_unknown"),
        "object": "chat.completion",
        "created": 0,
        "model": resp.get("model", requested_model),
        "choices": [{"index": 0, "message": message, "finish_reason": finish_reason}],
        "usage": {
            "prompt_tokens": usage.get("input_tokens", 0),
            "completion_tokens": usage.get("output_tokens", 0),
            "total_tokens": usage.get("input_tokens", 0) + usage.get("output_tokens", 0),
        },
    }


async def handle(request):
    if request.path != "/v1/chat/completions" or request.method != "POST":
        return web.Response(status=404, text="only POST /v1/chat/completions supported")

    raw = await request.read()
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as e:
        return web.Response(status=400, text=f"invalid JSON: {e}")

    requested_model = payload.get("model", "claude-sonnet-4-6")
    messages = payload.get("messages", [])
    tools = payload.get("tools")
    max_tokens = payload.get("max_tokens", 8192)
    is_stream_request = bool(payload.get("stream"))
    if max_tokens <= THINKING_BUDGET:
        max_tokens = THINKING_BUDGET + 4096

    system, anth_messages = messages_oai_to_anthropic(messages)

    anth_payload = {
        "model": requested_model,
        "max_tokens": max_tokens,
        "messages": anth_messages,
        "thinking": {"type": "enabled", "budget_tokens": THINKING_BUDGET},
    }
    if system:
        anth_payload["system"] = system
    if tools:
        anth_payload["tools"] = openai_tools_to_anthropic(tools)
        tc = payload.get("tool_choice")
        if tc == "auto":
            anth_payload["tool_choice"] = {"type": "auto"}
        elif isinstance(tc, dict) and tc.get("type") == "function":
            anth_payload["tool_choice"] = {"type": "tool", "name": tc["function"]["name"]}

    headers = {
        "x-api-key": UPSTREAM_API_KEY,
        "anthropic-version": ANTHROPIC_VERSION,
        "content-type": "application/json",
    }

    print(f"[anth-proxy] -> {requested_model} msgs={len(anth_messages)} tools={len(tools or [])}",
          file=sys.stderr, flush=True)

    timeout = aiohttp.ClientTimeout(total=600)
    async with aiohttp.ClientSession(timeout=timeout) as session:
        async with session.post(ANTHROPIC_URL, json=anth_payload, headers=headers) as upstream:
            resp_text = await upstream.text()
            if upstream.status != 200:
                print(f"[anth-proxy] upstream {upstream.status}: {resp_text[:500]}",
                      file=sys.stderr, flush=True)
                return web.Response(status=upstream.status, text=resp_text,
                                    content_type="application/json")
            try:
                anth_resp = json.loads(resp_text)
            except json.JSONDecodeError:
                return web.Response(status=502, text="upstream returned non-JSON")
            oai = anthropic_response_to_openai(anth_resp, requested_model)
            blocks = [b.get("type") for b in anth_resp.get("content", [])]
            stop = anth_resp.get("stop_reason")
            msg = oai["choices"][0]["message"]
            has_text = bool(msg.get("content"))
            has_tc = bool(msg.get("tool_calls"))
            print(f"[anth-proxy] upstream stop={stop} blocks={blocks} oai-text={has_text} oai-tools={has_tc}",
                  file=sys.stderr, flush=True)
            if not has_text and not has_tc:
                print(f"[anth-proxy] EMPTY RESPONSE — full upstream body:\n{resp_text[:2000]}",
                      file=sys.stderr, flush=True)
            if is_stream_request:
                sse_body = openai_response_to_sse_chunks(oai)
                return web.Response(body=sse_body, status=200,
                                    headers={"Content-Type": "text/event-stream",
                                             "Cache-Control": "no-cache"})
            return web.json_response(oai)


def main():
    app = web.Application(client_max_size=1024 * 1024 * 32)
    app.router.add_route("*", "/{path:.*}", handle)
    print(f"[anth-proxy] listening on 0.0.0.0:{PROXY_PORT} -> {ANTHROPIC_URL}", file=sys.stderr)
    web.run_app(app, host="0.0.0.0", port=PROXY_PORT, print=None)


if __name__ == "__main__":
    main()
