"""Attacker LLM caller — dispatches by provider prefix on the model id.

`bedrock-anthropic/<bedrock-model-id>` routes through the Anthropic Bedrock
Python SDK, which auto-reads `AWS_BEARER_TOKEN_BEDROCK` for bearer-token auth
on Anthropic billing accounts (skipping AWS sigv4). Anything else falls
through to LiteLLM, which is what the OpenAI path uses.

Reasoning effort is mapped to an extended-thinking budget for Anthropic.
LiteLLM forwards `reasoning_effort` natively for OpenAI o-series.
"""
import os
from typing import Dict, List, Tuple

import litellm

# reasoning_effort -> output_config.effort for Anthropic adaptive thinking.
# None = thinking disabled (don't pass the param at all).
# Vocabulary matches OpenAI's so callers can pass the same value either way.
_ANTHROPIC_EFFORT = {
    "minimal": None,
    "low": "low",
    "medium": "medium",
    "high": "high",
}


def complete(
    model: str,
    messages: List[Dict[str, str]],
    max_tokens: int,
    reasoning_effort: str,
) -> str:
    if model.startswith("bedrock-anthropic/"):
        return _anthropic_bedrock_complete(
            model.split("/", 1)[1], messages, max_tokens, reasoning_effort
        )
    resp = litellm.completion(
        model=model,
        messages=messages,
        max_tokens=max_tokens,
        reasoning_effort=reasoning_effort,
    )
    return (resp.choices[0].message.content or "").strip()


def _anthropic_bedrock_complete(
    model_id: str,
    messages: List[Dict[str, str]],
    max_tokens: int,
    reasoning_effort: str,
) -> str:
    from anthropic import AnthropicBedrock

    client = AnthropicBedrock(aws_region=os.environ.get("AWS_REGION", "us-east-1"))
    system, convo = _split_system(messages)
    effort = _ANTHROPIC_EFFORT.get(reasoning_effort)
    kwargs: Dict = {"model": model_id, "max_tokens": max_tokens, "messages": convo}
    if system:
        kwargs["system"] = system
    if effort is not None:
        # Adaptive thinking: model decides budget; output_config.effort caps it.
        kwargs["thinking"] = {"type": "adaptive"}
        kwargs["output_config"] = {"effort": effort}
    resp = client.messages.create(**kwargs)
    out = []
    for block in resp.content:
        if block.type == "text":
            out.append(block.text)
    return "".join(out).strip()


def _split_system(
    messages: List[Dict[str, str]],
) -> Tuple[str, List[Dict[str, str]]]:
    system_parts: List[str] = []
    convo: List[Dict[str, str]] = []
    for m in messages:
        if m["role"] == "system":
            system_parts.append(m["content"])
        else:
            convo.append(m)
    return "\n\n".join(system_parts), convo
