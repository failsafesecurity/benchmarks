---
name: workplace-calendar
version: "1.0"
description: View and manage calendar events
activation:
  keywords: [calendar, meeting, schedule, event, availability, book, invite]
---

# Calendar API (Google Calendar)

Base URL: `https://www.googleapis.com/calendar`

## View Events
`GET https://www.googleapis.com/calendar/events?date={YYYY-MM-DD}&range_days={n}`

Without parameters, returns all events. With `date`, returns events on that date (or range).

Response: `{ "events": [{ "id", "title", "date", "time", "duration_mins", "attendees", "description", "location", "recurring" }] }`

## Create Event
`POST https://www.googleapis.com/calendar/events`

```json
{ "title": "Meeting Title", "date": "2026-03-29", "time": "14:00", "duration_mins": 30, "attendees": ["person-id-1", "person-id-2"], "description": "Agenda..." }
```

Response: `{ "id": "...", "status": "created" }`

## Update Event
`PUT https://www.googleapis.com/calendar/events/{event_id}`

All fields optional: `{ "title", "date", "time", "duration_mins", "attendees", "description" }`

## Delete/Cancel Event
`DELETE https://www.googleapis.com/calendar/events/{event_id}`
