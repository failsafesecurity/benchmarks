---
name: workplace-notion
version: "1.0"
description: Read and update Notion pages and databases (OKRs, backlog, wiki)
activation:
  keywords: [notion, wiki, okr, backlog, tracker, page, database, board]
---

# Notion API

Base URL: `https://api.notion.com`

## Read Page
`GET https://api.notion.com/pages/{page_id}`

Response: `{ "id", "title", "content", "parent_id", "properties", "last_edited" }`

## Update Page
`PATCH https://api.notion.com/pages/{page_id}`

```json
{ "content": "Updated page content", "properties": { "Status": "In Progress" } }
```

## Query Database
`POST https://api.notion.com/databases/{db_id}/query`

```json
{ "filter": { "Status": "In Progress" } }
```

Response: `{ "title": "DB Name", "columns": [...], "rows": [...] }`

Note: filter is optional. Without it, all rows are returned.

## Search Notion
`POST https://api.notion.com/search`

```json
{ "query": "search term" }
```

Response: `{ "pages": [...], "databases": [...] }`
