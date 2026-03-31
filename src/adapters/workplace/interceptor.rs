use std::sync::Arc;

use async_trait::async_trait;
use ironclaw::llm::recording::{HttpExchangeRequest, HttpExchangeResponse, HttpInterceptor};
use tokio::sync::Mutex;

use super::server::HttpRequestLog;

/// Routes agent HTTP calls from real-looking API URLs (e.g., `https://api.slack.com/...`)
/// to the local mock web server.
///
/// The agent's skills reference real service URLs. This interceptor matches those
/// URLs and returns responses from the mock server instead of making real requests.
/// All other URLs pass through unintercepted.
#[derive(Debug)]
pub struct WorkplaceInterceptor {
    /// The mock server's actual base URL (e.g., `http://127.0.0.1:54321`).
    mock_base_url: String,
    /// URL prefixes to intercept, mapped to their mock path prefix.
    /// e.g., ("https://api.slack.com" -> "/api/slack")
    intercept_rules: Vec<InterceptRule>,
    /// HTTP client to forward requests to mock server.
    client: reqwest::Client,
    /// Log of all intercepted requests (method + path).
    request_log: Arc<Mutex<Vec<HttpRequestLog>>>,
}

#[derive(Debug, Clone)]
struct InterceptRule {
    /// Real URL prefix to match (e.g., "https://api.slack.com")
    real_prefix: String,
    /// Mock server path prefix to map to (e.g., "/api/slack")
    mock_prefix: String,
}

impl WorkplaceInterceptor {
    pub fn new(mock_base_url: String) -> Self {
        let intercept_rules = vec![
            InterceptRule {
                real_prefix: "https://slack.com/api".into(),
                mock_prefix: "/api/slack".into(),
            },
            InterceptRule {
                real_prefix: "https://www.googleapis.com/gmail".into(),
                mock_prefix: "/api/email".into(),
            },
            InterceptRule {
                real_prefix: "https://www.googleapis.com/calendar".into(),
                mock_prefix: "/api/calendar".into(),
            },
            InterceptRule {
                real_prefix: "https://docs.googleapis.com".into(),
                mock_prefix: "/api/docs".into(),
            },
            InterceptRule {
                real_prefix: "https://api.notion.com".into(),
                mock_prefix: "/api/notion".into(),
            },
            InterceptRule {
                real_prefix: "https://sheets.googleapis.com".into(),
                mock_prefix: "/api/sheets".into(),
            },
        ];

        Self {
            mock_base_url,
            intercept_rules,
            client: reqwest::Client::new(),
            request_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get a copy of the intercepted request log.
    pub async fn request_log(&self) -> Vec<HttpRequestLog> {
        self.request_log.lock().await.clone()
    }

    /// Find a matching intercept rule for the given URL.
    fn match_rule<'a>(&'a self, url: &'a str) -> Option<(&'a InterceptRule, &'a str)> {
        for rule in &self.intercept_rules {
            if let Some(rest) = url.strip_prefix(&rule.real_prefix) {
                return Some((rule, rest));
            }
        }
        None
    }
}

#[async_trait]
impl HttpInterceptor for WorkplaceInterceptor {
    async fn before_request(
        &self,
        request: &HttpExchangeRequest,
    ) -> Option<HttpExchangeResponse> {
        let (rule, rest) = self.match_rule(&request.url)?;

        // Build mock URL: base + mock_prefix + rest of path
        let mock_url = format!("{}{}{}", self.mock_base_url, rule.mock_prefix, rest);

        // Log the request
        let log_path = format!("{}{}", rule.mock_prefix, rest);
        self.request_log.lock().await.push(HttpRequestLog {
            method: request.method.clone(),
            path: log_path,
            timestamp: chrono::Utc::now(),
        });

        // Forward to mock server
        let method = reqwest::Method::from_bytes(request.method.as_bytes())
            .unwrap_or(reqwest::Method::GET);
        let mut req = self.client.request(method, &mock_url);
        for (k, v) in &request.headers {
            req = req.header(k.as_str(), v.as_str());
        }
        if let Some(ref body) = request.body {
            req = req
                .header("content-type", "application/json")
                .body(body.clone());
        }

        match req.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let headers: Vec<(String, String)> = resp
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();
                let body = resp.text().await.unwrap_or_default();
                Some(HttpExchangeResponse {
                    status,
                    headers,
                    body,
                })
            }
            Err(e) => Some(HttpExchangeResponse {
                status: 502,
                headers: vec![],
                body: format!("{{\"error\": \"Mock server error: {e}\"}}"),
            }),
        }
    }

    async fn after_response(
        &self,
        _request: &HttpExchangeRequest,
        _response: &HttpExchangeResponse,
    ) {
        // No-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_matching() {
        let interceptor = WorkplaceInterceptor::new("http://127.0.0.1:9999".into());

        // Slack
        let (rule, rest) = interceptor
            .match_rule("https://slack.com/api/channels/general/messages")
            .unwrap();
        assert_eq!(rule.mock_prefix, "/api/slack");
        assert_eq!(rest, "/channels/general/messages");

        // Notion
        let (rule, rest) = interceptor
            .match_rule("https://api.notion.com/pages/abc123")
            .unwrap();
        assert_eq!(rule.mock_prefix, "/api/notion");
        assert_eq!(rest, "/pages/abc123");

        // Non-matching URL
        assert!(interceptor.match_rule("https://example.com/foo").is_none());
    }
}
