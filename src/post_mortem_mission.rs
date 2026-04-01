use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::BenchError;

/// Event types that can trigger post-mortem analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeploymentEvent {
    /// Deployment failed with error details
    Failure {
        deployment_id: String,
        error_message: String,
        pipeline_stage: String,
        timestamp: String,
        log_path: Option<String>,
    },
    /// Deployment succeeded (for comparison)
    Success {
        deployment_id: String,
        timestamp: String,
    },
    /// Deployment started
    Started {
        deployment_id: String,
        timestamp: String,
    },
}

/// Root cause category for analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RootCauseCategory {
    CodeError,
    ConfigurationIssue,
    DependencyFailure,
    InfrastructureProblem,
    NetworkIssue,
    ResourceExhaustion,
    TestFailure,
    Unknown,
}

/// Suggested fix for identified root cause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedFix {
    pub category: RootCauseCategory,
    pub description: String,
    pub confidence: f64, // 0.0 to 1.0
    pub action_items: Vec<String>,
    pub related_logs: Vec<String>,
}

/// Post-mortem analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMortemReport {
    pub event: DeploymentEvent,
    pub analysis_timestamp: String,
    pub root_causes: Vec<SuggestedFix>,
    pub error_patterns: Vec<String>,
    pub recommendations: Vec<String>,
    pub severity: SeverityLevel,
    pub affected_services: Vec<String>,
}

/// Severity level of the deployment failure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Mission state for post-mortem analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PostMortemState {
    Idle,
    Analyzing,
    ReportGenerated,
    Archived,
}

/// Event-triggered post-mortem analysis mission
///
/// This mission automatically runs when a deployment failure event is detected.
/// It analyzes error logs and suggests root causes with actionable recommendations.
#[derive(Clone)]
pub struct PostMortemMission {
    pub id: String,
    pub state: Arc<tokio::sync::RwLock<PostMortemState>>,
    pub log_directory: PathBuf,
    pub analysis_config: AnalysisConfig,
    pub event_history: Arc<tokio::sync::RwLock<Vec<DeploymentEvent>>>,
}

/// Configuration for post-mortem analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub max_log_lines: usize,
    pub lookback_hours: u64,
    pub enable_pattern_matching: bool,
    pub confidence_threshold: f64,
    pub webhook_url: Option<String>,
    pub notify_on_critical: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            max_log_lines: 10000,
            lookback_hours: 24,
            enable_pattern_matching: true,
            confidence_threshold: 0.6,
            webhook_url: None,
            notify_on_critical: true,
        }
    }
}

impl PostMortemMission {
    /// Create a new post-mortem analysis mission
    pub fn new(id: &str, log_directory: &str) -> Self {
        Self {
            id: id.to_string(),
            state: Arc::new(tokio::sync::RwLock::new(PostMortemState::Idle)),
            log_directory: PathBuf::from(log_directory),
            analysis_config: AnalysisConfig::default(),
            event_history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Create with custom configuration
    pub fn with_config(id: &str, log_directory: &str, config: AnalysisConfig) -> Self {
        Self {
            id: id.to_string(),
            state: Arc::new(tokio::sync::RwLock::new(PostMortemState::Idle)),
            log_directory: PathBuf::from(log_directory),
            analysis_config: config,
            event_history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Get mission ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get current state
    pub async fn state(&self) -> PostMortemState {
        self.state.read().await.clone()
    }

    /// Record an event and trigger analysis if it's a failure
    pub async fn handle_event(
        &self,
        event: DeploymentEvent,
    ) -> Result<PostMortemReport, BenchError> {
        // Record event in history
        {
            let mut history = self.event_history.write().await;
            history.push(event.clone());
        }

        match &event {
            DeploymentEvent::Failure { .. } => {
                // Trigger post-mortem analysis
                self.set_state(PostMortemState::Analyzing).await;
                let report = self.analyze_failure(event).await?;
                self.set_state(PostMortemState::ReportGenerated).await;

                // Send webhook notification if configured
                if let Some(webhook) = &self.analysis_config.webhook_url {
                    self.send_webhook(&report, webhook).await?;
                }

                Ok(report)
            }
            _ => {
                // Non-failure events don't trigger analysis
                Ok(PostMortemReport {
                    event,
                    analysis_timestamp: chrono::Utc::now().to_rfc3339(),
                    root_causes: vec![],
                    error_patterns: vec![],
                    recommendations: vec![],
                    severity: SeverityLevel::Low,
                    affected_services: vec![],
                })
            }
        }
    }

    /// Set mission state
    async fn set_state(&self, state: PostMortemState) {
        let mut current = self.state.write().await;
        *current = state.clone();
        tracing::info!(
            "Post-mortem mission {} state changed to {:?}",
            self.id,
            state
        );
    }

    /// Analyze a deployment failure event
    async fn analyze_failure(
        &self,
        event: DeploymentEvent,
    ) -> Result<PostMortemReport, BenchError> {
        tracing::info!("Analyzing deployment failure for mission {}", self.id);

        let (_deployment_id, error_message, log_path) = match &event {
            DeploymentEvent::Failure {
                deployment_id,
                error_message,
                log_path,
                ..
            } => (
                deployment_id.clone(),
                error_message.clone(),
                log_path.clone(),
            ),
            _ => return Err(BenchError::Config("Expected failure event".to_string())),
        };

        // Read and parse logs if available
        let log_content = if let Some(path) = &log_path {
            self.read_logs(path).await.unwrap_or_default()
        } else {
            String::new()
        };

        // Perform root cause analysis
        let root_causes = self
            .identify_root_causes(&error_message, &log_content)
            .await;
        let error_patterns = self.extract_error_patterns(&log_content).await;
        let recommendations = self
            .generate_recommendations(&root_causes, &error_patterns)
            .await;
        let severity = self.calculate_severity(&error_message, &root_causes);
        let affected_services = self.identify_affected_services(&error_message, &log_content);

        Ok(PostMortemReport {
            event,
            analysis_timestamp: chrono::Utc::now().to_rfc3339(),
            root_causes,
            error_patterns,
            recommendations,
            severity,
            affected_services,
        })
    }

    /// Read logs from a file or directory
    async fn read_logs(&self, log_path: &str) -> Result<String, BenchError> {
        let path = PathBuf::from(log_path);

        if path.is_file() {
            let content = fs::read_to_string(&path)
                .await
                .map_err(|e| BenchError::Config(format!("Failed to read log file: {}", e)))?;
            Ok(content
                .lines()
                .take(self.analysis_config.max_log_lines)
                .collect::<Vec<_>>()
                .join("\n"))
        } else if path.is_dir() {
            // Read most recent log files from directory
            let mut logs = Vec::new();
            let mut entries = fs::read_dir(&path)
                .await
                .map_err(|e| BenchError::Config(format!("Failed to read log directory: {}", e)))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| BenchError::Config(format!("Failed to read directory entry: {}", e)))?
            {
                let entry_path: std::path::PathBuf = entry.path();
                if entry_path.is_file() {
                    if let Ok(content) = fs::read_to_string(&entry_path).await {
                        logs.push(content);
                    }
                }
            }

            Ok(logs.join("\n"))
        } else {
            Ok(String::new())
        }
    }

    /// Identify root causes from error message and logs
    async fn identify_root_causes(
        &self,
        error_message: &str,
        log_content: &str,
    ) -> Vec<SuggestedFix> {
        let mut fixes = Vec::new();

        // Pattern matching for common deployment failures
        let patterns: HashMap<&str, (RootCauseCategory, &str, Vec<String>)> = [
            (
                "connection refused",
                (
                    RootCauseCategory::NetworkIssue,
                    "Service connectivity failure",
                    vec![
                        "Check if the target service is running".to_string(),
                        "Verify network policies and firewalls".to_string(),
                        "Confirm service endpoints are correct".to_string(),
                    ],
                ),
            ),
            (
                "timeout",
                (
                    RootCauseCategory::InfrastructureProblem,
                    "Request timeout detected",
                    vec![
                        "Increase timeout thresholds if appropriate".to_string(),
                        "Check system resource utilization".to_string(),
                        "Review service performance metrics".to_string(),
                    ],
                ),
            ),
            (
                "permission denied",
                (
                    RootCauseCategory::ConfigurationIssue,
                    "Permission or access control issue",
                    vec![
                        "Verify IAM roles and permissions".to_string(),
                        "Check file system permissions".to_string(),
                        "Review service account configurations".to_string(),
                    ],
                ),
            ),
            (
                "out of memory",
                (
                    RootCauseCategory::ResourceExhaustion,
                    "Memory resource exhaustion",
                    vec![
                        "Increase memory limits for the service".to_string(),
                        "Check for memory leaks in the application".to_string(),
                        "Review resource allocation policies".to_string(),
                    ],
                ),
            ),
            (
                "failed to compile",
                (
                    RootCauseCategory::CodeError,
                    "Build or compilation error",
                    vec![
                        "Review recent code changes".to_string(),
                        "Check dependency versions".to_string(),
                        "Run local build tests".to_string(),
                    ],
                ),
            ),
            (
                "test failed",
                (
                    RootCauseCategory::TestFailure,
                    "Test suite failure",
                    vec![
                        "Review failing test logs".to_string(),
                        "Check test environment configuration".to_string(),
                        "Verify test data integrity".to_string(),
                    ],
                ),
            ),
            (
                "dependency",
                (
                    RootCauseCategory::DependencyFailure,
                    "External dependency failure",
                    vec![
                        "Check dependency service health".to_string(),
                        "Review dependency version compatibility".to_string(),
                        "Verify network access to dependency endpoints".to_string(),
                    ],
                ),
            ),
        ]
        .iter()
        .cloned()
        .collect();

        // Check error message and logs against patterns
        let combined_text = format!(
            "{} {}",
            error_message.to_lowercase(),
            log_content.to_lowercase()
        );

        for (pattern, (category, description, actions)) in &patterns {
            if combined_text.contains(pattern) {
                fixes.push(SuggestedFix {
                    category: category.clone(),
                    description: description.to_string(),
                    confidence: 0.8,
                    action_items: actions.clone(),
                    related_logs: self.find_related_log_lines(log_content, pattern).await,
                });
            }
        }

        // If no patterns matched, provide generic analysis
        if fixes.is_empty() {
            fixes.push(SuggestedFix {
                category: RootCauseCategory::Unknown,
                description: "Unable to automatically identify root cause".to_string(),
                confidence: 0.3,
                action_items: vec![
                    "Review full error logs manually".to_string(),
                    "Check deployment configuration".to_string(),
                    "Verify service health metrics".to_string(),
                    "Consider rollback if impact is severe".to_string(),
                ],
                related_logs: vec![],
            });
        }

        // Filter by confidence threshold
        fixes.retain(|fix| fix.confidence >= self.analysis_config.confidence_threshold);

        fixes
    }

    /// Find log lines related to a pattern
    async fn find_related_log_lines(&self, log_content: &str, pattern: &str) -> Vec<String> {
        log_content
            .lines()
            .filter(|line| line.to_lowercase().contains(pattern))
            .take(5)
            .map(|s| s.to_string())
            .collect()
    }

    /// Extract error patterns from logs
    async fn extract_error_patterns(&self, log_content: &str) -> Vec<String> {
        if !self.analysis_config.enable_pattern_matching {
            return vec![];
        }

        let mut patterns = Vec::new();
        let error_keywords = ["error", "fail", "exception", "critical", "fatal"];

        for line in log_content.lines() {
            let lower_line = line.to_lowercase();
            for keyword in &error_keywords {
                let line_str = line.to_string();
                if lower_line.contains(keyword) && !patterns.contains(&line_str) {
                    patterns.push(line_str);
                    if patterns.len() >= 20 {
                        return patterns;
                    }
                }
            }
        }

        patterns
    }

    /// Generate recommendations based on root causes
    async fn generate_recommendations(
        &self,
        root_causes: &[SuggestedFix],
        _error_patterns: &[String],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Category-specific recommendations
        for cause in root_causes {
            match cause.category {
                RootCauseCategory::NetworkIssue => {
                    recommendations
                        .push("Implement circuit breakers for external service calls".to_string());
                    recommendations.push("Add health checks for dependent services".to_string());
                }
                RootCauseCategory::ConfigurationIssue => {
                    recommendations
                        .push("Use configuration validation in CI/CD pipeline".to_string());
                    recommendations.push("Implement secrets management best practices".to_string());
                }
                RootCauseCategory::ResourceExhaustion => {
                    recommendations.push("Set up resource monitoring and alerting".to_string());
                    recommendations.push("Implement auto-scaling policies".to_string());
                }
                RootCauseCategory::CodeError => {
                    recommendations.push("Enhance static analysis in CI pipeline".to_string());
                    recommendations.push("Add more comprehensive unit tests".to_string());
                }
                RootCauseCategory::TestFailure => {
                    recommendations.push("Review test flakiness and add retries".to_string());
                    recommendations.push("Ensure test environment matches production".to_string());
                }
                RootCauseCategory::DependencyFailure => {
                    recommendations.push("Implement dependency version pinning".to_string());
                    recommendations
                        .push("Add fallback mechanisms for critical dependencies".to_string());
                }
                RootCauseCategory::InfrastructureProblem | RootCauseCategory::Unknown => {
                    recommendations.push("Review infrastructure capacity and limits".to_string());
                    recommendations.push("Implement better observability and logging".to_string());
                }
            }
        }

        // Deduplicate recommendations
        recommendations.sort();
        recommendations.dedup();

        recommendations
    }

    /// Calculate severity based on error and root causes
    fn calculate_severity(
        &self,
        error_message: &str,
        root_causes: &[SuggestedFix],
    ) -> SeverityLevel {
        // Check for critical keywords
        let lower_error = error_message.to_lowercase();
        if lower_error.contains("critical")
            || lower_error.contains("fatal")
            || lower_error.contains("panic")
        {
            return SeverityLevel::Critical;
        }

        if lower_error.contains("cluster")
            || lower_error.contains("database")
            || lower_error.contains("primary")
        {
            return SeverityLevel::High;
        }

        // Base severity on root cause confidence
        let max_confidence = root_causes
            .iter()
            .map(|r| r.confidence)
            .fold(0.0f64, f64::max);

        if max_confidence > 0.8 {
            SeverityLevel::High
        } else if max_confidence > 0.5 {
            SeverityLevel::Medium
        } else {
            SeverityLevel::Low
        }
    }

    /// Identify affected services from error message and logs
    fn identify_affected_services(&self, error_message: &str, log_content: &str) -> Vec<String> {
        let mut services = Vec::new();
        let combined = format!("{} {}", error_message, log_content);

        // Common service naming patterns
        let patterns = vec![
            "api-service",
            "web-frontend",
            "database",
            "auth-service",
            "payment-service",
            "notification-service",
            "cache",
            "queue",
            "worker",
        ];

        for pattern in patterns {
            if combined.to_lowercase().contains(pattern) {
                services.push(pattern.to_string());
            }
        }

        if services.is_empty() {
            services.push("unknown".to_string());
        }

        services
    }

    /// Send webhook notification for critical failures
    async fn send_webhook(
        &self,
        report: &PostMortemReport,
        webhook_url: &str,
    ) -> Result<(), BenchError> {
        if report.severity != SeverityLevel::Critical && !self.analysis_config.notify_on_critical {
            return Ok(());
        }

        let payload = serde_json::json!({
            "mission_id": self.id,
            "report": report,
            "notification_type": "post_mortem_analysis",
        });

        // In a real implementation, this would make an HTTP POST request
        tracing::info!("Would send webhook to {}: {:?}", webhook_url, payload);

        Ok(())
    }

    /// Archive a completed post-mortem report
    pub async fn archive_report(&self, report: &PostMortemReport) -> Result<PathBuf, BenchError> {
        let archive_dir = self.log_directory.join("archived");
        fs::create_dir_all(&archive_dir)
            .await
            .map_err(|e| BenchError::Config(format!("Failed to create archive dir: {}", e)))?;

        let filename = format!(
            "{}-{}.json",
            self.id,
            report.analysis_timestamp.replace(':', "-")
        );
        let archive_path = archive_dir.join(&filename);

        let json = serde_json::to_string_pretty(report)
            .map_err(|e| BenchError::Config(format!("Failed to serialize report: {}", e)))?;

        fs::write(&archive_path, json.as_bytes())
            .await
            .map_err(|e| BenchError::Config(format!("Failed to write archive: {}", e)))?;

        self.set_state(PostMortemState::Archived).await;

        Ok(archive_path)
    }

    /// Get event history
    pub async fn get_event_history(&self) -> Vec<DeploymentEvent> {
        self.event_history.read().await.clone()
    }

    /// Clear event history
    pub async fn clear_history(&self) {
        let mut history = self.event_history.write().await;
        history.clear();
    }
}

/// Event-triggered post-mortem analysis handler
///
/// This struct manages the event-driven mission creation and execution
pub struct PostMortemEventHandler {
    log_directory: PathBuf,
    config: AnalysisConfig,
    active_missions: Arc<tokio::sync::RwLock<HashMap<String, PostMortemMission>>>,
}

impl PostMortemEventHandler {
    /// Create a new event handler
    pub fn new(log_directory: &str) -> Self {
        Self {
            log_directory: PathBuf::from(log_directory),
            config: AnalysisConfig::default(),
            active_missions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Create with custom configuration
    pub fn with_config(log_directory: &str, config: AnalysisConfig) -> Self {
        Self {
            log_directory: PathBuf::from(log_directory),
            config,
            active_missions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Handle a deployment event - creates mission if needed and processes event
    pub async fn handle_deployment_event(
        &self,
        event: DeploymentEvent,
    ) -> Result<PostMortemReport, BenchError> {
        let deployment_id = match &event {
            DeploymentEvent::Failure { deployment_id, .. } => deployment_id.clone(),
            DeploymentEvent::Success { deployment_id, .. } => deployment_id.clone(),
            DeploymentEvent::Started { deployment_id, .. } => deployment_id.clone(),
        };

        // Get or create mission for this deployment
        let mission = {
            let mut missions = self.active_missions.write().await;
            missions
                .entry(deployment_id.clone())
                .or_insert_with(|| {
                    let mission_id = format!("postmortem-{}", deployment_id);
                    PostMortemMission::with_config(
                        &mission_id,
                        self.log_directory.to_str().unwrap_or("./logs"),
                        self.config.clone(),
                    )
                })
                .clone()
        };

        // Process the event through the mission
        mission.handle_event(event).await
    }

    /// Listen for deployment failure events (event-driven trigger)
    ///
    /// This is the main entry point for OnEvent cadence type missions
    pub async fn listen_for_failures(&self) -> Result<(), BenchError> {
        tracing::info!("Post-mortem handler listening for deployment failures...");

        // In a real implementation, this would subscribe to an event bus or message queue
        // For now, we provide the interface for external event triggering

        Ok(())
    }

    /// Get all active missions
    pub async fn get_active_missions(&self) -> Vec<String> {
        let missions = self.active_missions.read().await;
        missions.keys().cloned().collect()
    }
}

/// Factory function to create a post-mortem mission from event data
pub fn create_post_mortem_mission_from_event(
    event: &DeploymentEvent,
    log_dir: &str,
) -> PostMortemMission {
    let mission_id = match event {
        DeploymentEvent::Failure { deployment_id, .. } => format!("postmortem-{}", deployment_id),
        _ => format!("postmortem-{}", chrono::Utc::now().timestamp()),
    };

    PostMortemMission::new(&mission_id, log_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_deployment_failure() {
        let temp_dir = std::env::temp_dir().join("postmortem_test");
        let handler = PostMortemEventHandler::new(temp_dir.to_str().unwrap());

        let event = DeploymentEvent::Failure {
            deployment_id: "deploy-123".to_string(),
            error_message: "Connection refused to database service".to_string(),
            pipeline_stage: "deployment".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            log_path: None,
        };

        let report = handler.handle_deployment_event(event).await.unwrap();

        assert!(!report.root_causes.is_empty());
        assert_eq!(
            report.root_causes[0].category,
            RootCauseCategory::NetworkIssue
        );
    }

    #[tokio::test]
    async fn test_severity_calculation() {
        let mission = PostMortemMission::new("test", "./logs");

        let critical_error = "Critical: Database cluster failed";
        let causes = vec![SuggestedFix {
            category: RootCauseCategory::InfrastructureProblem,
            description: "Test".to_string(),
            confidence: 0.9,
            action_items: vec![],
            related_logs: vec![],
        }];

        let severity = mission.calculate_severity(critical_error, &causes);
        assert_eq!(severity, SeverityLevel::Critical);
    }

    #[test]
    fn test_event_serialization() {
        let event = DeploymentEvent::Failure {
            deployment_id: "test-1".to_string(),
            error_message: "Test error".to_string(),
            pipeline_stage: "build".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            log_path: Some("/var/logs/test.log".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: DeploymentEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, parsed);
    }
}
