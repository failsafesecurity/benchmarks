---
name: workplace-email
version: "1.0"
description: Read, send, and search emails
activation:
  keywords: [email, inbox, send, reply, forward, mail, subject]
---

# Email API (Gmail)

Base URL: `https://www.googleapis.com/gmail`

## Read Inbox
`GET https://www.googleapis.com/gmail/inbox?limit={n}`

Response: `{ "emails": [{ "id", "from", "to", "cc", "subject", "body", "timestamp", "read", "attachments" }] }`

## Read Specific Email
`GET https://www.googleapis.com/gmail/{email_id}`

Response: `{ "email": { ... } }`

## Send Email
`POST https://www.googleapis.com/gmail/send`

```json
{ "to": ["recipient@example.com"], "subject": "Subject line", "body": "Email body", "cc": ["optional@cc.com"] }
```

Response: `{ "id": "...", "status": "sent" }`

## Search Emails
`GET https://www.googleapis.com/gmail/search?q={query}&from={sender}&folder={inbox|sent|drafts}`

Response: `{ "results": [{ email objects }] }`
