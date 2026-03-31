use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Core company model
// ---------------------------------------------------------------------------

/// Complete state of a simulated company, seeded per scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyState {
    pub company: CompanyProfile,
    pub people: Vec<Person>,
    #[serde(default)]
    pub okrs: Vec<OKR>,
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub slack: SlackState,
    #[serde(default)]
    pub email: EmailState,
    #[serde(default)]
    pub calendar: CalendarState,
    #[serde(default)]
    pub docs: Vec<Document>,
    #[serde(default)]
    pub notion: NotionState,
    #[serde(default)]
    pub spreadsheets: Vec<Spreadsheet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyProfile {
    pub name: String,
    #[serde(default)]
    pub stage: String,
    #[serde(default)]
    pub headcount: u32,
    #[serde(default)]
    pub industry: String,
    #[serde(default)]
    pub founded: String,
    #[serde(default)]
    pub revenue_arr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub department: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub slack_handle: String,
    #[serde(default)]
    pub reports_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OKR {
    pub objective: String,
    #[serde(default)]
    pub key_results: Vec<String>,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub progress: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub sprint: Option<u32>,
    #[serde(default)]
    pub eta: Option<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub description: String,
}

// ---------------------------------------------------------------------------
// Slack
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlackState {
    #[serde(default)]
    pub channels: Vec<SlackChannel>,
    #[serde(default)]
    pub direct_messages: Vec<SlackDM>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackChannel {
    pub name: String,
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub messages: Vec<SlackMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackDM {
    /// The other person's ID.
    pub person_id: String,
    #[serde(default)]
    pub messages: Vec<SlackMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessage {
    pub id: String,
    pub from: String,
    #[serde(default)]
    pub from_name: String,
    pub content: String,
    pub timestamp: String,
    #[serde(default)]
    pub thread_id: Option<String>,
    #[serde(default)]
    pub reactions: Vec<String>,
}

// ---------------------------------------------------------------------------
// Email
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmailState {
    #[serde(default)]
    pub inbox: Vec<Email>,
    #[serde(default)]
    pub sent: Vec<Email>,
    #[serde(default)]
    pub drafts: Vec<Email>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,
    #[serde(default)]
    pub cc: Vec<String>,
    pub subject: String,
    pub body: String,
    pub timestamp: String,
    #[serde(default)]
    pub read: bool,
    #[serde(default)]
    pub attachments: Vec<String>,
}

// ---------------------------------------------------------------------------
// Calendar
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CalendarState {
    #[serde(default)]
    pub events: Vec<CalendarEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub date: String,
    pub time: String,
    pub duration_mins: u32,
    #[serde(default)]
    pub attendees: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub recurring: bool,
}

// ---------------------------------------------------------------------------
// Documents (Google Docs-like)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub last_modified: String,
    #[serde(default)]
    pub shared_with: Vec<String>,
}

// ---------------------------------------------------------------------------
// Notion
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotionState {
    #[serde(default)]
    pub pages: Vec<NotionPage>,
    #[serde(default)]
    pub databases: Vec<NotionDatabase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionPage {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub properties: serde_json::Value,
    #[serde(default)]
    pub last_edited: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionDatabase {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub columns: Vec<NotionColumn>,
    #[serde(default)]
    pub rows: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionColumn {
    pub name: String,
    #[serde(default = "default_column_type")]
    pub column_type: String,
}

fn default_column_type() -> String {
    "text".to_string()
}

// ---------------------------------------------------------------------------
// Spreadsheets
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spreadsheet {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub sheets: Vec<Sheet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheet {
    pub name: String,
    /// Row-major data: each inner Vec is a row of cell values.
    #[serde(default)]
    pub data: Vec<Vec<serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// State injection (applied between turns)
// ---------------------------------------------------------------------------

/// Describes mutations to apply to CompanyState between turns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateInjection {
    /// Append messages to a Slack channel. Key format: channel name (e.g., "#leadership").
    #[serde(default)]
    pub slack_channel_messages: std::collections::HashMap<String, Vec<SlackMessage>>,
    /// Append emails to inbox.
    #[serde(default)]
    pub inbox_emails: Vec<Email>,
    /// Add calendar events.
    #[serde(default)]
    pub calendar_events: Vec<CalendarEvent>,
    /// Add/update documents.
    #[serde(default)]
    pub docs: Vec<Document>,
}

impl StateInjection {
    /// Apply this injection to a mutable CompanyState.
    pub fn apply(&self, state: &mut CompanyState) {
        // Slack channel messages
        for (channel_name, messages) in &self.slack_channel_messages {
            let normalized = if channel_name.starts_with('#') {
                channel_name.clone()
            } else {
                format!("#{channel_name}")
            };
            if let Some(ch) = state
                .slack
                .channels
                .iter_mut()
                .find(|c| c.name == normalized || c.name == *channel_name)
            {
                ch.messages.extend(messages.iter().cloned());
            } else {
                // Create channel if it doesn't exist
                state.slack.channels.push(SlackChannel {
                    name: normalized,
                    members: vec![],
                    messages: messages.clone(),
                });
            }
        }

        // Inbox emails
        state.email.inbox.extend(self.inbox_emails.iter().cloned());

        // Calendar events
        state
            .calendar
            .events
            .extend(self.calendar_events.iter().cloned());

        // Documents (add or update by ID)
        for doc in &self.docs {
            if let Some(existing) = state.docs.iter_mut().find(|d| d.id == doc.id) {
                *existing = doc.clone();
            } else {
                state.docs.push(doc.clone());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Expected state for scoring
// ---------------------------------------------------------------------------

/// Expected state after scenario completion, used for assertion-based scoring.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpectedState {
    /// Expected Slack messages that were sent (posted) by the agent.
    #[serde(default)]
    pub slack_messages_sent: Vec<ExpectedSlackMessage>,
    /// Expected emails that were sent by the agent.
    #[serde(default)]
    pub emails_sent: Vec<ExpectedEmail>,
    /// Expected calendar events that were created.
    #[serde(default)]
    pub calendar_events_created: Vec<ExpectedCalendarEvent>,
    /// Expected documents that were created or updated.
    #[serde(default)]
    pub docs_created: Vec<ExpectedDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedSlackMessage {
    pub channel: String,
    #[serde(default)]
    pub content_contains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedEmail {
    #[serde(default)]
    pub to_contains: Option<String>,
    #[serde(default)]
    pub subject_contains: Option<String>,
    #[serde(default)]
    pub body_contains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedCalendarEvent {
    #[serde(default)]
    pub title_contains: Option<String>,
    #[serde(default)]
    pub attendees_include: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedDocument {
    #[serde(default)]
    pub title_contains: Option<String>,
    #[serde(default)]
    pub content_contains: Vec<String>,
}

// ---------------------------------------------------------------------------
// Template merging
// ---------------------------------------------------------------------------

impl CompanyState {
    /// Merge scenario-specific overrides into a base template.
    /// Override fields replace template fields when present.
    pub fn merge_overrides(&mut self, overrides: CompanyStateOverrides) {
        if let Some(slack) = overrides.slack {
            // Merge channels: override channels replace same-named template channels
            for override_ch in slack.channels {
                if let Some(existing) = self
                    .slack
                    .channels
                    .iter_mut()
                    .find(|c| c.name == override_ch.name)
                {
                    existing.messages.extend(override_ch.messages);
                    if !override_ch.members.is_empty() {
                        existing.members = override_ch.members;
                    }
                } else {
                    self.slack.channels.push(override_ch);
                }
            }
            self.slack
                .direct_messages
                .extend(slack.direct_messages);
        }

        if let Some(email) = overrides.email {
            self.email.inbox.extend(email.inbox);
            self.email.sent.extend(email.sent);
            self.email.drafts.extend(email.drafts);
        }

        if let Some(calendar) = overrides.calendar {
            self.calendar.events.extend(calendar.events);
        }

        if let Some(docs) = overrides.docs {
            for doc in docs {
                if let Some(existing) = self.docs.iter_mut().find(|d| d.id == doc.id) {
                    *existing = doc;
                } else {
                    self.docs.push(doc);
                }
            }
        }

        if let Some(notion) = overrides.notion {
            self.notion.pages.extend(notion.pages);
            self.notion.databases.extend(notion.databases);
        }

        if let Some(spreadsheets) = overrides.spreadsheets {
            for sheet in spreadsheets {
                if let Some(existing) = self.spreadsheets.iter_mut().find(|s| s.id == sheet.id) {
                    *existing = sheet;
                } else {
                    self.spreadsheets.push(sheet);
                }
            }
        }
    }
}

/// Partial overrides for CompanyState. All fields optional.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompanyStateOverrides {
    #[serde(default)]
    pub slack: Option<SlackState>,
    #[serde(default)]
    pub email: Option<EmailState>,
    #[serde(default)]
    pub calendar: Option<CalendarState>,
    #[serde(default)]
    pub docs: Option<Vec<Document>>,
    #[serde(default)]
    pub notion: Option<NotionState>,
    #[serde(default)]
    pub spreadsheets: Option<Vec<Spreadsheet>>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_injection_appends_slack_messages() {
        let mut state = CompanyState {
            company: CompanyProfile {
                name: "Test".into(),
                stage: String::new(),
                headcount: 0,
                industry: String::new(),
                founded: String::new(),
                revenue_arr: String::new(),
            },
            people: vec![],
            okrs: vec![],
            projects: vec![],
            slack: SlackState {
                channels: vec![SlackChannel {
                    name: "#general".into(),
                    members: vec![],
                    messages: vec![SlackMessage {
                        id: "1".into(),
                        from: "alice".into(),
                        from_name: "Alice".into(),
                        content: "Hello".into(),
                        timestamp: "2026-03-29T08:00:00Z".into(),
                        thread_id: None,
                        reactions: vec![],
                    }],
                }],
                direct_messages: vec![],
            },
            email: EmailState::default(),
            calendar: CalendarState::default(),
            docs: vec![],
            notion: NotionState::default(),
            spreadsheets: vec![],
        };

        let injection = StateInjection {
            slack_channel_messages: [(
                "#general".to_string(),
                vec![SlackMessage {
                    id: "2".into(),
                    from: "bob".into(),
                    from_name: "Bob".into(),
                    content: "Urgent!".into(),
                    timestamp: "2026-03-29T09:00:00Z".into(),
                    thread_id: None,
                    reactions: vec![],
                }],
            )]
            .into(),
            ..Default::default()
        };

        injection.apply(&mut state);
        assert_eq!(state.slack.channels[0].messages.len(), 2);
        assert_eq!(state.slack.channels[0].messages[1].content, "Urgent!");
    }

    #[test]
    fn test_state_injection_creates_new_channel() {
        let mut state = CompanyState {
            company: CompanyProfile {
                name: "Test".into(),
                stage: String::new(),
                headcount: 0,
                industry: String::new(),
                founded: String::new(),
                revenue_arr: String::new(),
            },
            people: vec![],
            okrs: vec![],
            projects: vec![],
            slack: SlackState::default(),
            email: EmailState::default(),
            calendar: CalendarState::default(),
            docs: vec![],
            notion: NotionState::default(),
            spreadsheets: vec![],
        };

        let injection = StateInjection {
            slack_channel_messages: [(
                "new-channel".to_string(),
                vec![SlackMessage {
                    id: "1".into(),
                    from: "alice".into(),
                    from_name: "Alice".into(),
                    content: "First message".into(),
                    timestamp: "2026-03-29T08:00:00Z".into(),
                    thread_id: None,
                    reactions: vec![],
                }],
            )]
            .into(),
            ..Default::default()
        };

        injection.apply(&mut state);
        assert_eq!(state.slack.channels.len(), 1);
        assert_eq!(state.slack.channels[0].name, "#new-channel");
    }

    #[test]
    fn test_expected_state_deserializes() {
        let json = r##"{
            "slack_messages_sent": [
                { "channel": "#engineering", "content_contains": ["API", "customer"] }
            ],
            "emails_sent": [
                { "to_contains": "cto@example.com", "subject_contains": "Urgent" }
            ]
        }"##;
        let expected: ExpectedState = serde_json::from_str(json).unwrap();
        assert_eq!(expected.slack_messages_sent.len(), 1);
        assert_eq!(expected.emails_sent.len(), 1);
    }

    #[test]
    fn test_company_state_round_trip() {
        let state = CompanyState {
            company: CompanyProfile {
                name: "Acme".into(),
                stage: "Series B".into(),
                headcount: 48,
                industry: "SaaS".into(),
                founded: "2022".into(),
                revenue_arr: "$3.2M".into(),
            },
            people: vec![Person {
                id: "alex-ceo".into(),
                name: "Alex Chen".into(),
                role: "CEO".into(),
                department: "Executive".into(),
                email: "alex@acme.tech".into(),
                slack_handle: "@alex".into(),
                reports_to: None,
            }],
            okrs: vec![],
            projects: vec![],
            slack: SlackState::default(),
            email: EmailState::default(),
            calendar: CalendarState::default(),
            docs: vec![],
            notion: NotionState::default(),
            spreadsheets: vec![],
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: CompanyState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.company.name, "Acme");
        assert_eq!(deserialized.people[0].id, "alex-ceo");
    }
}
