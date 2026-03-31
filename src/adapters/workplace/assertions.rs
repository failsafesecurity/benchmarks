use regex::Regex;
use serde::{Deserialize, Serialize};

use super::server::HttpRequestLog;
use super::state::{CompanyState, ExpectedState};
use crate::suite::TaskSubmission;

// ---------------------------------------------------------------------------
// Workplace-specific assertions
// ---------------------------------------------------------------------------

/// Multi-criterion assertions for a workplace scenario turn.
///
/// Extends the standard assertions with HTTP-aware checks and state verification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkplaceAssertions {
    // -- Standard response assertions --
    #[serde(default)]
    pub response_contains: Vec<String>,
    #[serde(default)]
    pub response_not_contains: Vec<String>,
    #[serde(default)]
    pub response_matches: Option<String>,

    // -- Tool-level assertions (http tool name) --
    #[serde(default)]
    pub tools_used: Vec<String>,
    #[serde(default)]
    pub tools_not_used: Vec<String>,
    #[serde(default)]
    pub min_tool_calls: Option<usize>,
    #[serde(default)]
    pub max_tool_calls: Option<usize>,

    // -- HTTP-aware assertions --
    /// URL patterns that must appear in the request log (e.g., "GET /api/slack/").
    #[serde(default)]
    pub http_calls_contain: Vec<String>,
    /// URL patterns that must NOT appear.
    #[serde(default)]
    pub http_calls_not_contain: Vec<String>,
    /// Minimum number of HTTP calls.
    #[serde(default)]
    pub min_http_calls: Option<usize>,
    /// Maximum number of HTTP calls.
    #[serde(default)]
    pub max_http_calls: Option<usize>,
    /// Ordered pairs of HTTP patterns: first must appear before second.
    /// Each entry is [before_pattern, after_pattern].
    #[serde(default)]
    pub http_call_order: Vec<(String, String)>,

    // -- Error handling --
    #[serde(default)]
    pub no_error: bool,
}

impl WorkplaceAssertions {
    /// Evaluate assertions against a submission and HTTP request log.
    /// Returns (score 0.0-1.0, list of failure descriptions).
    pub fn evaluate(
        &self,
        submission: &TaskSubmission,
        http_log: &[HttpRequestLog],
    ) -> (f64, Vec<String>) {
        let mut passed: usize = 0;
        let mut total: usize = 0;
        let mut failures: Vec<String> = Vec::new();

        let response_lower = submission.response.to_lowercase();

        // -- no_error --
        if self.no_error {
            total += 1;
            if submission.error.is_some() {
                failures.push(format!(
                    "no_error: task errored: {}",
                    submission.error.as_deref().unwrap_or("unknown")
                ));
            } else {
                passed += 1;
            }
        }

        // -- response_contains --
        for needle in &self.response_contains {
            total += 1;
            if response_lower.contains(&needle.to_lowercase()) {
                passed += 1;
            } else {
                failures.push(format!("response_contains: missing \"{needle}\""));
            }
        }

        // -- response_not_contains --
        for needle in &self.response_not_contains {
            total += 1;
            if response_lower.contains(&needle.to_lowercase()) {
                failures.push(format!("response_not_contains: found \"{needle}\""));
            } else {
                passed += 1;
            }
        }

        // -- response_matches --
        if let Some(ref pattern) = self.response_matches {
            total += 1;
            match Regex::new(pattern) {
                Ok(re) => {
                    if re.is_match(&submission.response) {
                        passed += 1;
                    } else {
                        failures.push(format!("response_matches: /{pattern}/ did not match"));
                    }
                }
                Err(e) => {
                    failures.push(format!("response_matches: bad regex: {e}"));
                }
            }
        }

        // -- tools_used / tools_not_used --
        let tool_set: std::collections::HashSet<&str> =
            submission.tool_calls.iter().map(|s| s.as_str()).collect();

        for tool in &self.tools_used {
            total += 1;
            if tool_set.contains(tool.as_str()) {
                passed += 1;
            } else {
                failures.push(format!("tools_used: \"{tool}\" not called"));
            }
        }

        for tool in &self.tools_not_used {
            total += 1;
            if tool_set.contains(tool.as_str()) {
                failures.push(format!("tools_not_used: \"{tool}\" was called"));
            } else {
                passed += 1;
            }
        }

        // -- min/max tool calls --
        if let Some(min) = self.min_tool_calls {
            total += 1;
            if submission.tool_calls.len() >= min {
                passed += 1;
            } else {
                failures.push(format!(
                    "min_tool_calls: expected >= {min}, got {}",
                    submission.tool_calls.len()
                ));
            }
        }

        if let Some(max) = self.max_tool_calls {
            total += 1;
            if submission.tool_calls.len() <= max {
                passed += 1;
            } else {
                failures.push(format!(
                    "max_tool_calls: expected <= {max}, got {}",
                    submission.tool_calls.len()
                ));
            }
        }

        // -- HTTP call assertions --
        let http_entries: Vec<String> = http_log
            .iter()
            .map(|r| format!("{} {}", r.method, r.path))
            .collect();

        for pattern in &self.http_calls_contain {
            total += 1;
            let pattern_lower = pattern.to_lowercase();
            if http_entries
                .iter()
                .any(|e| e.to_lowercase().contains(&pattern_lower))
            {
                passed += 1;
            } else {
                failures.push(format!("http_calls_contain: no call matching \"{pattern}\""));
            }
        }

        for pattern in &self.http_calls_not_contain {
            total += 1;
            let pattern_lower = pattern.to_lowercase();
            if http_entries
                .iter()
                .any(|e| e.to_lowercase().contains(&pattern_lower))
            {
                failures.push(format!(
                    "http_calls_not_contain: found call matching \"{pattern}\""
                ));
            } else {
                passed += 1;
            }
        }

        if let Some(min) = self.min_http_calls {
            total += 1;
            if http_log.len() >= min {
                passed += 1;
            } else {
                failures.push(format!(
                    "min_http_calls: expected >= {min}, got {}",
                    http_log.len()
                ));
            }
        }

        if let Some(max) = self.max_http_calls {
            total += 1;
            if http_log.len() <= max {
                passed += 1;
            } else {
                failures.push(format!(
                    "max_http_calls: expected <= {max}, got {}",
                    http_log.len()
                ));
            }
        }

        // -- HTTP call ordering --
        for (before, after) in &self.http_call_order {
            total += 1;
            let before_lower = before.to_lowercase();
            let after_lower = after.to_lowercase();
            let before_pos = http_entries
                .iter()
                .position(|e| e.to_lowercase().contains(&before_lower));
            let after_pos = http_entries
                .iter()
                .position(|e| e.to_lowercase().contains(&after_lower));
            match (before_pos, after_pos) {
                (Some(b), Some(a)) if b < a => {
                    passed += 1;
                }
                (Some(_), Some(_)) => {
                    failures.push(format!(
                        "http_call_order: \"{before}\" should come before \"{after}\""
                    ));
                }
                (None, _) => {
                    failures.push(format!(
                        "http_call_order: \"{before}\" not found in request log"
                    ));
                }
                (_, None) => {
                    failures.push(format!(
                        "http_call_order: \"{after}\" not found in request log"
                    ));
                }
            }
        }

        if total == 0 {
            return (1.0, failures);
        }

        (passed as f64 / total as f64, failures)
    }
}

// ---------------------------------------------------------------------------
// State-based scoring
// ---------------------------------------------------------------------------

/// Evaluate expected state assertions against the actual server state.
/// Returns (score 0.0-1.0, list of failure descriptions).
pub fn evaluate_expected_state(
    actual: &CompanyState,
    expected: &ExpectedState,
) -> (f64, Vec<String>) {
    let mut passed: usize = 0;
    let mut total: usize = 0;
    let mut failures: Vec<String> = Vec::new();

    // -- Expected Slack messages sent --
    for expected_msg in &expected.slack_messages_sent {
        total += 1;
        let channel_name = &expected_msg.channel;
        let found = actual
            .slack
            .channels
            .iter()
            .find(|c| c.name == *channel_name || c.name == format!("#{channel_name}"))
            .map(|c| {
                c.messages.iter().any(|msg| {
                    msg.from == "assistant"
                        && expected_msg.content_contains.iter().all(|needle| {
                            msg.content.to_lowercase().contains(&needle.to_lowercase())
                        })
                })
            })
            .unwrap_or(false);
        if found {
            passed += 1;
        } else {
            failures.push(format!(
                "slack_messages_sent: no agent message in {} containing {:?}",
                channel_name, expected_msg.content_contains
            ));
        }
    }

    // -- Expected emails sent --
    for expected_email in &expected.emails_sent {
        total += 1;
        let found = actual.email.sent.iter().any(|e| {
            let to_match = expected_email
                .to_contains
                .as_ref()
                .is_none_or(|tc| e.to.iter().any(|t| t.to_lowercase().contains(&tc.to_lowercase())));
            let subj_match = expected_email
                .subject_contains
                .as_ref()
                .is_none_or(|sc| e.subject.to_lowercase().contains(&sc.to_lowercase()));
            let body_match = expected_email
                .body_contains
                .iter()
                .all(|bc| e.body.to_lowercase().contains(&bc.to_lowercase()));
            to_match && subj_match && body_match
        });
        if found {
            passed += 1;
        } else {
            failures.push(format!(
                "emails_sent: no sent email matching to={:?} subject={:?}",
                expected_email.to_contains, expected_email.subject_contains
            ));
        }
    }

    // -- Expected calendar events created --
    for expected_event in &expected.calendar_events_created {
        total += 1;
        let found = actual.calendar.events.iter().any(|e| {
            let title_match = expected_event
                .title_contains
                .as_ref()
                .is_none_or(|tc| e.title.to_lowercase().contains(&tc.to_lowercase()));
            let attendees_match = expected_event
                .attendees_include
                .iter()
                .all(|a| e.attendees.iter().any(|ea| ea.to_lowercase().contains(&a.to_lowercase())));
            title_match && attendees_match
        });
        if found {
            passed += 1;
        } else {
            failures.push(format!(
                "calendar_events_created: no event matching title={:?} attendees={:?}",
                expected_event.title_contains, expected_event.attendees_include
            ));
        }
    }

    // -- Expected docs created --
    for expected_doc in &expected.docs_created {
        total += 1;
        let found = actual.docs.iter().any(|d| {
            let title_match = expected_doc
                .title_contains
                .as_ref()
                .is_none_or(|tc| d.title.to_lowercase().contains(&tc.to_lowercase()));
            let content_match = expected_doc
                .content_contains
                .iter()
                .all(|cc| d.content.to_lowercase().contains(&cc.to_lowercase()));
            title_match && content_match
        });
        if found {
            passed += 1;
        } else {
            failures.push(format!(
                "docs_created: no doc matching title={:?}",
                expected_doc.title_contains
            ));
        }
    }

    if total == 0 {
        return (1.0, failures);
    }

    (passed as f64 / total as f64, failures)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suite::TaskSubmission;

    fn make_submission(response: &str, tool_calls: Vec<&str>) -> TaskSubmission {
        TaskSubmission {
            response: response.to_string(),
            conversation: vec![],
            tool_calls: tool_calls.into_iter().map(|s| s.to_string()).collect(),
            error: None,
        }
    }

    fn make_log(entries: Vec<(&str, &str)>) -> Vec<HttpRequestLog> {
        entries
            .into_iter()
            .map(|(method, path)| HttpRequestLog {
                method: method.to_string(),
                path: path.to_string(),
                timestamp: chrono::Utc::now(),
            })
            .collect()
    }

    #[test]
    fn test_response_contains_pass() {
        let a = WorkplaceAssertions {
            response_contains: vec!["budget".into(), "overrun".into()],
            ..Default::default()
        };
        let sub = make_submission("The budget is overrun by 15%", vec![]);
        let (score, failures) = a.evaluate(&sub, &[]);
        assert_eq!(score, 1.0);
        assert!(failures.is_empty());
    }

    #[test]
    fn test_http_calls_contain() {
        let a = WorkplaceAssertions {
            http_calls_contain: vec!["GET /api/slack/".into(), "POST /api/email/send".into()],
            ..Default::default()
        };
        let log = make_log(vec![
            ("GET", "/api/slack/channels/general/messages"),
            ("POST", "/api/email/send"),
        ]);
        let sub = make_submission("Done", vec!["http"]);
        let (score, _) = a.evaluate(&sub, &log);
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_http_call_order() {
        let a = WorkplaceAssertions {
            http_call_order: vec![("GET /api/slack/".into(), "POST /api/email/send".into())],
            ..Default::default()
        };
        // Correct order
        let log = make_log(vec![
            ("GET", "/api/slack/channels/general/messages"),
            ("POST", "/api/email/send"),
        ]);
        let sub = make_submission("Done", vec![]);
        let (score, _) = a.evaluate(&sub, &log);
        assert_eq!(score, 1.0);

        // Wrong order
        let log = make_log(vec![
            ("POST", "/api/email/send"),
            ("GET", "/api/slack/channels/general/messages"),
        ]);
        let (score, failures) = a.evaluate(&sub, &log);
        assert_eq!(score, 0.0);
        assert!(failures[0].contains("should come before"));
    }

    #[test]
    fn test_min_http_calls() {
        let a = WorkplaceAssertions {
            min_http_calls: Some(3),
            ..Default::default()
        };
        let log = make_log(vec![("GET", "/api/slack/channels/general/messages")]);
        let sub = make_submission("Done", vec![]);
        let (score, failures) = a.evaluate(&sub, &log);
        assert_eq!(score, 0.0);
        assert!(failures[0].contains("min_http_calls"));
    }

    #[test]
    fn test_empty_assertions_pass() {
        let a = WorkplaceAssertions::default();
        let sub = make_submission("anything", vec![]);
        let (score, _) = a.evaluate(&sub, &[]);
        assert_eq!(score, 1.0);
    }
}
