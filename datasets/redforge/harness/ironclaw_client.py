"""Signed HTTP client for the Ironclaw webhook endpoint."""
import hashlib
import hmac
import json
import time
from dataclasses import dataclass
from typing import Optional

import httpx


@dataclass
class IronclawReply:
    message_id: str
    status: str
    response: Optional[str]
    latency_s: float


class IronclawClient:
    def __init__(self, url: str, secret: str, timeout_s: float = 180.0):
        self.url = url
        self.secret = secret.encode()
        self.timeout_s = timeout_s

    def send(
        self,
        content: str,
        thread_id: Optional[str] = None,
        wait_for_response: bool = True,
    ) -> IronclawReply:
        body_obj = {"content": content, "wait_for_response": wait_for_response}
        if thread_id is not None:
            body_obj["thread_id"] = thread_id
        # Sign the exact bytes we POST — whitespace drift invalidates HMAC.
        body_bytes = json.dumps(body_obj, separators=(",", ":")).encode()
        sig = hmac.new(self.secret, body_bytes, hashlib.sha256).hexdigest()
        headers = {
            "Content-Type": "application/json",
            "X-Hub-Signature-256": f"sha256={sig}",
        }
        t0 = time.perf_counter()
        resp = httpx.post(
            self.url, content=body_bytes, headers=headers, timeout=self.timeout_s
        )
        resp.raise_for_status()
        data = resp.json()
        return IronclawReply(
            message_id=data["message_id"],
            status=data["status"],
            response=data.get("response"),
            latency_s=time.perf_counter() - t0,
        )
