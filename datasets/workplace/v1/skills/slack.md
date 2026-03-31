---
name: workplace-slack
version: "1.0"
description: Read and send Slack messages in the company workspace
activation:
  keywords: [slack, message, channel, dm, post, notify, team, thread]
---

# Slack API

Base URL: `https://slack.com/api`

## Read Channel Messages
`GET https://slack.com/api/channels/{channel_name}/messages?limit={n}`

Returns recent messages from a channel (newest first).

Available channels: `leadership`, `engineering`, `marketing`, `finance`, `product`, `general`

Response:
```json
{ "messages": [{ "id": "...", "from": "person-id", "from_name": "Full Name", "content": "...", "timestamp": "ISO8601", "thread_id": null, "reactions": [] }] }
```

## Send Message to Channel
`POST https://slack.com/api/channels/{channel_name}/messages`

```json
{ "content": "Your message here", "thread_id": "optional-thread-id" }
```

Response: `{ "id": "...", "timestamp": "..." }`

## Search Messages
`GET https://slack.com/api/search?q={query}&channel={channel}&from={person_id}`

Search across all channels. Only `q` is required.

Response: `{ "results": [{ "channel": "#channel-name", "message": { ... } }] }`

## Direct Messages
`GET https://slack.com/api/dm/{person_id}/messages?limit={n}` — Read DMs with a person
`POST https://slack.com/api/dm/{person_id}/messages` — Send DM: `{ "content": "..." }`
