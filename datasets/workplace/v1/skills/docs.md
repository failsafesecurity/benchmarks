---
name: workplace-docs
version: "1.0"
description: Create, read, update, and search documents
activation:
  keywords: [document, doc, write, draft, report, brief, memo, strategy]
---

# Documents API (Google Docs)

Base URL: `https://docs.googleapis.com`

## List Documents
`GET https://docs.googleapis.com/`

Response: `{ "documents": [{ "id", "title", "owner", "last_modified" }] }`

## Read Document
`GET https://docs.googleapis.com/{doc_id}`

Response: `{ "id", "title", "content", "owner", "last_modified", "shared_with" }`

## Create Document
`POST https://docs.googleapis.com/`

```json
{ "title": "Document Title", "content": "Full document content..." }
```

Response: `{ "id": "...", "status": "created" }`

## Update Document
`PUT https://docs.googleapis.com/{doc_id}`

```json
{ "title": "New Title", "content": "Updated content" }
```

## Search Documents
`GET https://docs.googleapis.com/search?q={query}`

Searches document titles and content.

Response: `{ "results": [{ document objects }] }`
