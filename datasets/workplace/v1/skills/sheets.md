---
name: workplace-sheets
version: "1.0"
description: Read and update spreadsheet data (budgets, metrics, dashboards)
activation:
  keywords: [spreadsheet, sheet, budget, metrics, cells, data, numbers, financial]
---

# Spreadsheets API (Google Sheets)

Base URL: `https://sheets.googleapis.com`

## Read Spreadsheet
`GET https://sheets.googleapis.com/{sheet_id}`

Returns full spreadsheet with all sheets.

Response: `{ "id", "name", "sheets": [{ "name": "Sheet1", "data": [[cell, cell, ...], ...] }] }`

## Read Cell Range
`GET https://sheets.googleapis.com/{sheet_id}/values/{sheet_name}`

Returns data from a specific sheet tab.

Response: `{ "sheet": "Q1", "values": [[...], ...] }`

## Update Cell Range
`PUT https://sheets.googleapis.com/{sheet_id}/values/{sheet_name}`

```json
{ "values": [["row1col1", "row1col2"], ["row2col1", "row2col2"]] }
```

Response: `{ "status": "updated", "sheet": "Q1" }`
