use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::state::*;

// ---------------------------------------------------------------------------
// Request log (for assertion evaluation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct HttpRequestLog {
    pub method: String,
    pub path: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// Shared server state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub company: Arc<Mutex<CompanyState>>,
    pub request_log: Arc<Mutex<Vec<HttpRequestLog>>>,
}

// ---------------------------------------------------------------------------
// WorkplaceServer
// ---------------------------------------------------------------------------

pub struct WorkplaceServer {
    state: AppState,
    addr: std::net::SocketAddr,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl WorkplaceServer {
    /// Start server on a random port, seeded with initial company state.
    pub async fn start(initial_state: CompanyState) -> Self {
        let state = AppState {
            company: Arc::new(Mutex::new(initial_state)),
            request_log: Arc::new(Mutex::new(Vec::new())),
        };
        let app = build_router(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind workplace mock server");
        let addr = listener.local_addr().unwrap();
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    shutdown_rx.await.ok();
                })
                .await
                .ok();
        });
        tracing::info!("Workplace mock server started on {}", addr);
        Self {
            state,
            addr,
            shutdown_tx: Some(shutdown_tx),
        }
    }

    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.addr.port())
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    /// Snapshot current company state for scoring.
    pub async fn snapshot(&self) -> CompanyState {
        self.state.company.lock().await.clone()
    }

    /// Apply a state injection (between turns).
    pub async fn inject_state(&self, injection: StateInjection) {
        let mut state = self.state.company.lock().await;
        injection.apply(&mut state);
    }

    /// Get the HTTP request log for assertion evaluation.
    pub async fn request_log(&self) -> Vec<HttpRequestLog> {
        self.state.request_log.lock().await.clone()
    }
}

impl Drop for WorkplaceServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            tx.send(()).ok();
        }
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

fn build_router(state: AppState) -> Router {
    Router::new()
        // Slack
        .route(
            "/api/slack/channels/{channel}/messages",
            get(slack_channel_messages).post(slack_send_channel),
        )
        .route("/api/slack/search", get(slack_search))
        .route(
            "/api/slack/dm/{person_id}/messages",
            get(slack_dm_messages).post(slack_send_dm),
        )
        // Email
        .route("/api/email/inbox", get(email_inbox))
        .route("/api/email/send", post(email_send))
        .route("/api/email/search", get(email_search))
        .route("/api/email/{email_id}", get(email_read_one))
        // Calendar
        .route(
            "/api/calendar/events",
            get(calendar_list).post(calendar_create),
        )
        .route(
            "/api/calendar/events/{event_id}",
            put(calendar_update).delete(calendar_delete),
        )
        // Docs
        .route("/api/docs", get(docs_list).post(docs_create))
        .route("/api/docs/search", get(docs_search))
        .route("/api/docs/{doc_id}", get(docs_read).put(docs_update))
        // Notion
        .route(
            "/api/notion/pages/{page_id}",
            get(notion_page_read).patch(notion_page_update),
        )
        .route(
            "/api/notion/databases/{db_id}/query",
            post(notion_db_query),
        )
        .route("/api/notion/search", post(notion_search))
        // Sheets
        .route("/api/sheets/{sheet_id}", get(sheets_read))
        .route(
            "/api/sheets/{sheet_id}/values/{range}",
            get(sheets_range_read).put(sheets_range_update),
        )
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Logging middleware helper
// ---------------------------------------------------------------------------

async fn log_request(state: &AppState, method: &str, path: &str) {
    state.request_log.lock().await.push(HttpRequestLog {
        method: method.to_string(),
        path: path.to_string(),
        timestamp: chrono::Utc::now(),
    });
}

// ---------------------------------------------------------------------------
// Query / request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct LimitQuery {
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct SlackSearchQuery {
    pub q: String,
    pub channel: Option<String>,
    pub from: Option<String>,
}

#[derive(Deserialize)]
pub struct EmailSearchQuery {
    pub q: String,
    pub from: Option<String>,
    pub folder: Option<String>,
}

#[derive(Deserialize)]
pub struct CalendarQuery {
    pub date: Option<String>,
    pub range_days: Option<u32>,
}

#[derive(Deserialize)]
pub struct DocSearchQuery {
    pub q: String,
}

#[derive(Deserialize)]
pub struct SendMessageBody {
    pub content: String,
    pub thread_id: Option<String>,
}

#[derive(Deserialize)]
pub struct SendEmailBody {
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
    #[serde(default)]
    pub cc: Vec<String>,
}

#[derive(Deserialize)]
pub struct CreateEventBody {
    pub title: String,
    pub date: String,
    pub time: String,
    pub duration_mins: u32,
    #[serde(default)]
    pub attendees: Vec<String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Deserialize)]
pub struct UpdateEventBody {
    pub title: Option<String>,
    pub date: Option<String>,
    pub time: Option<String>,
    pub duration_mins: Option<u32>,
    pub attendees: Option<Vec<String>>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateDocBody {
    pub title: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateDocBody {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub struct NotionPageUpdateBody {
    pub content: Option<String>,
    pub properties: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct NotionDbQueryBody {
    pub filter: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct NotionSearchBody {
    pub query: String,
}

#[derive(Deserialize)]
pub struct SheetUpdateBody {
    pub values: Vec<Vec<serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------


// ---------------------------------------------------------------------------
// Slack handlers
// ---------------------------------------------------------------------------

async fn slack_channel_messages(
    State(state): State<AppState>,
    Path(channel): Path<String>,
    Query(params): Query<LimitQuery>,
) -> Json<serde_json::Value> {
    log_request(&state, "GET", &format!("/api/slack/channels/{channel}/messages")).await;
    let s = state.company.lock().await;
    let limit = params.limit.unwrap_or(20);
    let messages = s
        .slack
        .channels
        .iter()
        .find(|c| c.name == format!("#{channel}") || c.name == channel)
        .map(|c| {
            c.messages
                .iter()
                .rev()
                .take(limit)
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Json(serde_json::json!({ "messages": messages }))
}

async fn slack_send_channel(
    State(state): State<AppState>,
    Path(channel): Path<String>,
    Json(body): Json<SendMessageBody>,
) -> Json<serde_json::Value> {
    log_request(&state, "POST", &format!("/api/slack/channels/{channel}/messages")).await;
    let mut s = state.company.lock().await;
    let msg = SlackMessage {
        id: uuid::Uuid::new_v4().to_string(),
        from: "assistant".into(),
        from_name: "AI Assistant".into(),
        content: body.content,
        timestamp: chrono::Utc::now().to_rfc3339(),
        thread_id: body.thread_id,
        reactions: vec![],
    };
    let ch_name = if channel.starts_with('#') {
        channel.clone()
    } else {
        format!("#{channel}")
    };
    if let Some(ch) = s
        .slack
        .channels
        .iter_mut()
        .find(|c| c.name == ch_name || c.name == channel)
    {
        ch.messages.push(msg.clone());
    }
    Json(serde_json::json!({ "id": msg.id, "timestamp": msg.timestamp }))
}

async fn slack_search(
    State(state): State<AppState>,
    Query(params): Query<SlackSearchQuery>,
) -> Json<serde_json::Value> {
    log_request(&state, "GET", "/api/slack/search").await;
    let s = state.company.lock().await;
    let q = params.q.to_lowercase();
    let mut results = Vec::new();
    for ch in &s.slack.channels {
        if let Some(ref filter_ch) = params.channel {
            let norm = if filter_ch.starts_with('#') {
                filter_ch.clone()
            } else {
                format!("#{filter_ch}")
            };
            if ch.name != norm && ch.name != *filter_ch {
                continue;
            }
        }
        for msg in &ch.messages {
            if let Some(ref from) = params.from {
                if msg.from != *from {
                    continue;
                }
            }
            if msg.content.to_lowercase().contains(&q) {
                results.push(serde_json::json!({
                    "channel": ch.name,
                    "message": msg,
                }));
            }
        }
    }
    Json(serde_json::json!({ "results": results }))
}

async fn slack_dm_messages(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Query(params): Query<LimitQuery>,
) -> Json<serde_json::Value> {
    log_request(&state, "GET", &format!("/api/slack/dm/{person_id}/messages")).await;
    let s = state.company.lock().await;
    let limit = params.limit.unwrap_or(20);
    let messages = s
        .slack
        .direct_messages
        .iter()
        .find(|dm| dm.person_id == person_id)
        .map(|dm| {
            dm.messages
                .iter()
                .rev()
                .take(limit)
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Json(serde_json::json!({ "messages": messages }))
}

async fn slack_send_dm(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(body): Json<SendMessageBody>,
) -> Json<serde_json::Value> {
    log_request(&state, "POST", &format!("/api/slack/dm/{person_id}/messages")).await;
    let mut s = state.company.lock().await;
    let msg = SlackMessage {
        id: uuid::Uuid::new_v4().to_string(),
        from: "assistant".into(),
        from_name: "AI Assistant".into(),
        content: body.content,
        timestamp: chrono::Utc::now().to_rfc3339(),
        thread_id: body.thread_id,
        reactions: vec![],
    };
    if let Some(dm) = s
        .slack
        .direct_messages
        .iter_mut()
        .find(|dm| dm.person_id == person_id)
    {
        dm.messages.push(msg.clone());
    } else {
        s.slack.direct_messages.push(SlackDM {
            person_id,
            messages: vec![msg.clone()],
        });
    }
    Json(serde_json::json!({ "id": msg.id, "timestamp": msg.timestamp }))
}

// ---------------------------------------------------------------------------
// Email handlers
// ---------------------------------------------------------------------------

async fn email_inbox(
    State(state): State<AppState>,
    Query(params): Query<LimitQuery>,
) -> Json<serde_json::Value> {
    log_request(&state, "GET", "/api/email/inbox").await;
    let s = state.company.lock().await;
    let limit = params.limit.unwrap_or(50);
    let emails: Vec<_> = s.email.inbox.iter().rev().take(limit).cloned().collect();
    Json(serde_json::json!({ "emails": emails }))
}

async fn email_read_one(
    State(state): State<AppState>,
    Path(email_id): Path<String>,
) -> Json<serde_json::Value> {
    log_request(&state, "GET", &format!("/api/email/{email_id}")).await;
    let s = state.company.lock().await;
    let email = s
        .email
        .inbox
        .iter()
        .chain(s.email.sent.iter())
        .find(|e| e.id == email_id);
    match email {
        Some(e) => Json(serde_json::json!({ "email": e })),
        None => Json(serde_json::json!({ "error": "Email not found" })),
    }
}

async fn email_send(
    State(state): State<AppState>,
    Json(body): Json<SendEmailBody>,
) -> Json<serde_json::Value> {
    log_request(&state, "POST", "/api/email/send").await;
    let mut s = state.company.lock().await;
    let email = Email {
        id: uuid::Uuid::new_v4().to_string(),
        from: "assistant@acme.tech".into(),
        to: body.to,
        cc: body.cc,
        subject: body.subject,
        body: body.body,
        timestamp: chrono::Utc::now().to_rfc3339(),
        read: true,
        attachments: vec![],
    };
    let id = email.id.clone();
    s.email.sent.push(email);
    Json(serde_json::json!({ "id": id, "status": "sent" }))
}

async fn email_search(
    State(state): State<AppState>,
    Query(params): Query<EmailSearchQuery>,
) -> Json<serde_json::Value> {
    log_request(&state, "GET", "/api/email/search").await;
    let s = state.company.lock().await;
    let q = params.q.to_lowercase();
    let folder = params.folder.as_deref().unwrap_or("inbox");
    let source = match folder {
        "sent" => &s.email.sent,
        "drafts" => &s.email.drafts,
        _ => &s.email.inbox,
    };
    let results: Vec<_> = source
        .iter()
        .filter(|e| {
            let matches_q = e.subject.to_lowercase().contains(&q)
                || e.body.to_lowercase().contains(&q)
                || e.from.to_lowercase().contains(&q);
            let matches_from = params
                .from
                .as_ref()
                .is_none_or(|f| e.from.to_lowercase().contains(&f.to_lowercase()));
            matches_q && matches_from
        })
        .cloned()
        .collect();
    Json(serde_json::json!({ "results": results }))
}

// ---------------------------------------------------------------------------
// Calendar handlers
// ---------------------------------------------------------------------------

async fn calendar_list(
    State(state): State<AppState>,
    Query(params): Query<CalendarQuery>,
) -> Json<serde_json::Value> {
    log_request(&state, "GET", "/api/calendar/events").await;
    let s = state.company.lock().await;
    let events: Vec<_> = if let Some(ref date) = params.date {
        let range = params.range_days.unwrap_or(1);
        s.calendar
            .events
            .iter()
            .filter(|e| {
                // Simple date range filter
                if range == 1 {
                    e.date == *date
                } else {
                    e.date >= *date // Simplified: just filter from start date
                }
            })
            .cloned()
            .collect()
    } else {
        s.calendar.events.clone()
    };
    Json(serde_json::json!({ "events": events }))
}

async fn calendar_create(
    State(state): State<AppState>,
    Json(body): Json<CreateEventBody>,
) -> Json<serde_json::Value> {
    log_request(&state, "POST", "/api/calendar/events").await;
    let mut s = state.company.lock().await;
    let event = CalendarEvent {
        id: uuid::Uuid::new_v4().to_string(),
        title: body.title,
        date: body.date,
        time: body.time,
        duration_mins: body.duration_mins,
        attendees: body.attendees,
        description: body.description,
        location: None,
        recurring: false,
    };
    let id = event.id.clone();
    s.calendar.events.push(event);
    Json(serde_json::json!({ "id": id, "status": "created" }))
}

async fn calendar_update(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(body): Json<UpdateEventBody>,
) -> Json<serde_json::Value> {
    log_request(&state, "PUT", &format!("/api/calendar/events/{event_id}")).await;
    let mut s = state.company.lock().await;
    if let Some(event) = s.calendar.events.iter_mut().find(|e| e.id == event_id) {
        if let Some(title) = body.title {
            event.title = title;
        }
        if let Some(date) = body.date {
            event.date = date;
        }
        if let Some(time) = body.time {
            event.time = time;
        }
        if let Some(mins) = body.duration_mins {
            event.duration_mins = mins;
        }
        if let Some(attendees) = body.attendees {
            event.attendees = attendees;
        }
        if let Some(desc) = body.description {
            event.description = desc;
        }
        Json(serde_json::json!({ "status": "updated" }))
    } else {
        Json(serde_json::json!({ "error": "Event not found" }))
    }
}

async fn calendar_delete(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Json<serde_json::Value> {
    log_request(&state, "DELETE", &format!("/api/calendar/events/{event_id}")).await;
    let mut s = state.company.lock().await;
    let before = s.calendar.events.len();
    s.calendar.events.retain(|e| e.id != event_id);
    if s.calendar.events.len() < before {
        Json(serde_json::json!({ "status": "deleted" }))
    } else {
        Json(serde_json::json!({ "error": "Event not found" }))
    }
}

// ---------------------------------------------------------------------------
// Docs handlers
// ---------------------------------------------------------------------------

async fn docs_list(State(state): State<AppState>) -> Json<serde_json::Value> {
    log_request(&state, "GET", "/api/docs").await;
    let s = state.company.lock().await;
    let docs: Vec<_> = s
        .docs
        .iter()
        .map(|d| {
            serde_json::json!({
                "id": d.id,
                "title": d.title,
                "owner": d.owner,
                "last_modified": d.last_modified,
            })
        })
        .collect();
    Json(serde_json::json!({ "documents": docs }))
}

async fn docs_create(
    State(state): State<AppState>,
    Json(body): Json<CreateDocBody>,
) -> Json<serde_json::Value> {
    log_request(&state, "POST", "/api/docs").await;
    let mut s = state.company.lock().await;
    let doc = Document {
        id: uuid::Uuid::new_v4().to_string(),
        title: body.title,
        content: body.content,
        owner: "assistant".into(),
        last_modified: chrono::Utc::now().to_rfc3339(),
        shared_with: vec![],
    };
    let id = doc.id.clone();
    s.docs.push(doc);
    Json(serde_json::json!({ "id": id, "status": "created" }))
}

async fn docs_read(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Json<serde_json::Value> {
    log_request(&state, "GET", &format!("/api/docs/{doc_id}")).await;
    let s = state.company.lock().await;
    match s.docs.iter().find(|d| d.id == doc_id) {
        Some(doc) => Json(serde_json::to_value(doc).unwrap()),
        None => Json(serde_json::json!({ "error": "Document not found" })),
    }
}

async fn docs_update(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(body): Json<UpdateDocBody>,
) -> Json<serde_json::Value> {
    log_request(&state, "PUT", &format!("/api/docs/{doc_id}")).await;
    let mut s = state.company.lock().await;
    if let Some(doc) = s.docs.iter_mut().find(|d| d.id == doc_id) {
        if let Some(title) = body.title {
            doc.title = title;
        }
        if let Some(content) = body.content {
            doc.content = content;
        }
        doc.last_modified = chrono::Utc::now().to_rfc3339();
        Json(serde_json::json!({ "status": "updated" }))
    } else {
        Json(serde_json::json!({ "error": "Document not found" }))
    }
}

async fn docs_search(
    State(state): State<AppState>,
    Query(params): Query<DocSearchQuery>,
) -> Json<serde_json::Value> {
    log_request(&state, "GET", "/api/docs/search").await;
    let s = state.company.lock().await;
    let q = params.q.to_lowercase();
    let results: Vec<_> = s
        .docs
        .iter()
        .filter(|d| {
            d.title.to_lowercase().contains(&q) || d.content.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();
    Json(serde_json::json!({ "results": results }))
}

// ---------------------------------------------------------------------------
// Notion handlers
// ---------------------------------------------------------------------------

async fn notion_page_read(
    State(state): State<AppState>,
    Path(page_id): Path<String>,
) -> Json<serde_json::Value> {
    log_request(&state, "GET", &format!("/api/notion/pages/{page_id}")).await;
    let s = state.company.lock().await;
    match s.notion.pages.iter().find(|p| p.id == page_id) {
        Some(page) => Json(serde_json::to_value(page).unwrap()),
        None => Json(serde_json::json!({ "error": "Page not found" })),
    }
}

async fn notion_page_update(
    State(state): State<AppState>,
    Path(page_id): Path<String>,
    Json(body): Json<NotionPageUpdateBody>,
) -> Json<serde_json::Value> {
    log_request(&state, "PATCH", &format!("/api/notion/pages/{page_id}")).await;
    let mut s = state.company.lock().await;
    if let Some(page) = s.notion.pages.iter_mut().find(|p| p.id == page_id) {
        if let Some(content) = body.content {
            page.content = content;
        }
        if let Some(properties) = body.properties {
            page.properties = properties;
        }
        page.last_edited = chrono::Utc::now().to_rfc3339();
        Json(serde_json::json!({ "status": "updated" }))
    } else {
        Json(serde_json::json!({ "error": "Page not found" }))
    }
}

async fn notion_db_query(
    State(state): State<AppState>,
    Path(db_id): Path<String>,
    Json(_body): Json<NotionDbQueryBody>,
) -> Json<serde_json::Value> {
    log_request(&state, "POST", &format!("/api/notion/databases/{db_id}/query")).await;
    let s = state.company.lock().await;
    match s.notion.databases.iter().find(|d| d.id == db_id) {
        Some(db) => Json(serde_json::json!({
            "title": db.title,
            "columns": db.columns,
            "rows": db.rows,
        })),
        None => Json(serde_json::json!({ "error": "Database not found" })),
    }
}

async fn notion_search(
    State(state): State<AppState>,
    Json(body): Json<NotionSearchBody>,
) -> Json<serde_json::Value> {
    log_request(&state, "POST", "/api/notion/search").await;
    let s = state.company.lock().await;
    let q = body.query.to_lowercase();
    let pages: Vec<_> = s
        .notion
        .pages
        .iter()
        .filter(|p| {
            p.title.to_lowercase().contains(&q) || p.content.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();
    let databases: Vec<_> = s
        .notion
        .databases
        .iter()
        .filter(|d| d.title.to_lowercase().contains(&q))
        .cloned()
        .collect();
    Json(serde_json::json!({ "pages": pages, "databases": databases }))
}

// ---------------------------------------------------------------------------
// Sheets handlers
// ---------------------------------------------------------------------------

async fn sheets_read(
    State(state): State<AppState>,
    Path(sheet_id): Path<String>,
) -> Json<serde_json::Value> {
    log_request(&state, "GET", &format!("/api/sheets/{sheet_id}")).await;
    let s = state.company.lock().await;
    match s.spreadsheets.iter().find(|sp| sp.id == sheet_id) {
        Some(sp) => Json(serde_json::to_value(sp).unwrap()),
        None => Json(serde_json::json!({ "error": "Spreadsheet not found" })),
    }
}

async fn sheets_range_read(
    State(state): State<AppState>,
    Path((sheet_id, range)): Path<(String, String)>,
) -> Json<serde_json::Value> {
    log_request(
        &state,
        "GET",
        &format!("/api/sheets/{sheet_id}/values/{range}"),
    )
    .await;
    let s = state.company.lock().await;
    match s.spreadsheets.iter().find(|sp| sp.id == sheet_id) {
        Some(sp) => {
            // Find sheet by name (range can be "SheetName" or "SheetName!A1:C5")
            let sheet_name = range.split('!').next().unwrap_or(&range);
            match sp.sheets.iter().find(|sh| sh.name == sheet_name) {
                Some(sheet) => Json(serde_json::json!({
                    "sheet": sheet.name,
                    "values": sheet.data,
                })),
                None => {
                    // If no sheet matches, return the first sheet's data
                    if let Some(sheet) = sp.sheets.first() {
                        Json(serde_json::json!({
                            "sheet": sheet.name,
                            "values": sheet.data,
                        }))
                    } else {
                        Json(serde_json::json!({ "error": "Sheet not found", "available_sheets": sp.sheets.iter().map(|s| &s.name).collect::<Vec<_>>() }))
                    }
                }
            }
        }
        None => Json(serde_json::json!({ "error": "Spreadsheet not found" })),
    }
}

async fn sheets_range_update(
    State(state): State<AppState>,
    Path((sheet_id, range)): Path<(String, String)>,
    Json(body): Json<SheetUpdateBody>,
) -> Json<serde_json::Value> {
    log_request(
        &state,
        "PUT",
        &format!("/api/sheets/{sheet_id}/values/{range}"),
    )
    .await;
    let mut s = state.company.lock().await;
    if let Some(sp) = s.spreadsheets.iter_mut().find(|sp| sp.id == sheet_id) {
        let sheet_name = range.split('!').next().unwrap_or(&range);
        if let Some(sheet) = sp.sheets.iter_mut().find(|sh| sh.name == sheet_name) {
            sheet.data = body.values;
            Json(serde_json::json!({ "status": "updated", "sheet": sheet_name }))
        } else {
            Json(serde_json::json!({ "error": "Sheet not found" }))
        }
    } else {
        Json(serde_json::json!({ "error": "Spreadsheet not found" }))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> CompanyState {
        CompanyState {
            company: CompanyProfile {
                name: "Test Corp".into(),
                stage: "Seed".into(),
                headcount: 10,
                industry: "Tech".into(),
                founded: "2025".into(),
                revenue_arr: "$500K".into(),
            },
            people: vec![Person {
                id: "alice".into(),
                name: "Alice".into(),
                role: "CEO".into(),
                department: "Executive".into(),
                email: "alice@test.com".into(),
                slack_handle: "@alice".into(),
                reports_to: None,
            }],
            okrs: vec![],
            projects: vec![],
            slack: SlackState {
                channels: vec![SlackChannel {
                    name: "#general".into(),
                    members: vec!["alice".into()],
                    messages: vec![SlackMessage {
                        id: "msg-1".into(),
                        from: "alice".into(),
                        from_name: "Alice".into(),
                        content: "Good morning team!".into(),
                        timestamp: "2026-03-29T08:00:00Z".into(),
                        thread_id: None,
                        reactions: vec![],
                    }],
                }],
                direct_messages: vec![],
            },
            email: EmailState {
                inbox: vec![Email {
                    id: "email-1".into(),
                    from: "partner@example.com".into(),
                    to: vec!["alice@test.com".into()],
                    cc: vec![],
                    subject: "Partnership Proposal".into(),
                    body: "Let's discuss a partnership.".into(),
                    timestamp: "2026-03-29T07:00:00Z".into(),
                    read: false,
                    attachments: vec![],
                }],
                sent: vec![],
                drafts: vec![],
            },
            calendar: CalendarState {
                events: vec![CalendarEvent {
                    id: "event-1".into(),
                    title: "Team Standup".into(),
                    date: "2026-03-29".into(),
                    time: "09:00".into(),
                    duration_mins: 30,
                    attendees: vec!["alice".into()],
                    description: "Daily standup".into(),
                    location: None,
                    recurring: true,
                }],
            },
            docs: vec![Document {
                id: "doc-1".into(),
                title: "Company Strategy".into(),
                content: "Our strategy is to grow.".into(),
                owner: "alice".into(),
                last_modified: "2026-03-28".into(),
                shared_with: vec![],
            }],
            notion: NotionState::default(),
            spreadsheets: vec![Spreadsheet {
                id: "sheet-1".into(),
                name: "Budget".into(),
                sheets: vec![Sheet {
                    name: "Q1".into(),
                    data: vec![
                        vec!["Department".into(), "Jan".into(), "Feb".into()],
                        vec!["Engineering".into(), 100000.into(), 105000.into()],
                    ],
                }],
            }],
        }
    }

    #[tokio::test]
    async fn test_server_starts_and_responds() {
        let server = WorkplaceServer::start(test_state()).await;
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/api/slack/channels/general/messages", server.base_url()))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["messages"][0]["content"], "Good morning team!");
    }

    #[tokio::test]
    async fn test_slack_send_mutates_state() {
        let server = WorkplaceServer::start(test_state()).await;
        let client = reqwest::Client::new();

        // Send a message
        client
            .post(format!(
                "{}/api/slack/channels/general/messages",
                server.base_url()
            ))
            .json(&serde_json::json!({ "content": "Hello from test!" }))
            .send()
            .await
            .unwrap();

        // Verify state
        let snapshot = server.snapshot().await;
        let general = snapshot
            .slack
            .channels
            .iter()
            .find(|c| c.name == "#general")
            .unwrap();
        assert_eq!(general.messages.len(), 2);
        assert_eq!(general.messages[1].content, "Hello from test!");
    }

    #[tokio::test]
    async fn test_email_send() {
        let server = WorkplaceServer::start(test_state()).await;
        let client = reqwest::Client::new();

        client
            .post(format!("{}/api/email/send", server.base_url()))
            .json(&serde_json::json!({
                "to": ["bob@example.com"],
                "subject": "Test",
                "body": "Hello Bob",
            }))
            .send()
            .await
            .unwrap();

        let snapshot = server.snapshot().await;
        assert_eq!(snapshot.email.sent.len(), 1);
        assert_eq!(snapshot.email.sent[0].to[0], "bob@example.com");
    }

    #[tokio::test]
    async fn test_request_log() {
        let server = WorkplaceServer::start(test_state()).await;
        let client = reqwest::Client::new();

        client
            .get(format!("{}/api/email/inbox", server.base_url()))
            .send()
            .await
            .unwrap();
        client
            .get(format!(
                "{}/api/slack/channels/general/messages",
                server.base_url()
            ))
            .send()
            .await
            .unwrap();

        let log = server.request_log().await;
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].method, "GET");
        assert_eq!(log[0].path, "/api/email/inbox");
        assert_eq!(log[1].path, "/api/slack/channels/general/messages");
    }

    #[tokio::test]
    async fn test_state_injection() {
        let server = WorkplaceServer::start(test_state()).await;

        server
            .inject_state(StateInjection {
                slack_channel_messages: [(
                    "#general".to_string(),
                    vec![SlackMessage {
                        id: "injected-1".into(),
                        from: "bob".into(),
                        from_name: "Bob".into(),
                        content: "Urgent update!".into(),
                        timestamp: "2026-03-29T10:00:00Z".into(),
                        thread_id: None,
                        reactions: vec![],
                    }],
                )]
                .into(),
                ..Default::default()
            })
            .await;

        let snapshot = server.snapshot().await;
        let general = snapshot
            .slack
            .channels
            .iter()
            .find(|c| c.name == "#general")
            .unwrap();
        assert_eq!(general.messages.len(), 2);
        assert_eq!(general.messages[1].content, "Urgent update!");
    }
}
