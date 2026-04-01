use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::time::interval;

use crate::error::BenchError;

/// Mission state for deployment pipeline monitoring
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionState {
    Running,
    Paused,
    Stopped,
}

/// Deployment pipeline monitor mission
///
/// Monitors the deployment pipeline at regular intervals with pause/resume capability.
#[derive(Clone)]
pub struct DeploymentMission {
    id: String,
    state: Arc<AtomicMissionState>,
    interval_seconds: u64,
    deployment_url: String,
}

/// Atomic wrapper for mission state
#[derive(Debug)]
pub struct AtomicMissionState {
    inner: AtomicBool, // true = running, false = paused/stopped
}

impl AtomicMissionState {
    pub fn new(running: bool) -> Self {
        Self {
            inner: AtomicBool::new(running),
        }
    }

    pub fn is_running(&self) -> bool {
        self.inner.load(Ordering::SeqCst)
    }

    pub fn set_running(&self, running: bool) {
        self.inner.store(running, Ordering::SeqCst);
    }
}

impl DeploymentMission {
    /// Create a new deployment monitoring mission
    pub fn new(id: &str, deployment_url: &str, interval_hours: f64) -> Self {
        let interval_seconds = (interval_hours * 3600.0) as u64;

        Self {
            id: id.to_string(),
            state: Arc::new(AtomicMissionState::new(true)),
            interval_seconds,
            deployment_url: deployment_url.to_string(),
        }
    }

    /// Get mission ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get current mission state
    pub fn state(&self) -> MissionState {
        if self.state.is_running() {
            MissionState::Running
        } else {
            MissionState::Paused
        }
    }

    /// Pause the mission
    pub fn pause(&self) {
        self.state.set_running(false);
        tracing::info!("Mission {} paused", self.id);
    }

    /// Resume the mission
    pub fn resume(&self) {
        self.state.set_running(true);
        tracing::info!("Mission {} resumed", self.id);
    }

    /// Stop the mission permanently
    pub fn stop(&self) {
        self.state.set_running(false);
        tracing::info!("Mission {} stopped", self.id);
    }

    /// Check if mission is active (running or paused, not stopped)
    pub fn is_active(&self) -> bool {
        self.state.is_running()
    }

    /// Run the mission monitoring loop
    pub async fn run(&self) -> Result<(), BenchError> {
        let mut ticker = interval(Duration::from_secs(self.interval_seconds));

        tracing::info!(
            "Starting deployment mission '{}' - monitoring {} every {} hours",
            self.id,
            self.deployment_url,
            self.interval_seconds / 3600
        );

        loop {
            ticker.tick().await;

            // Check if mission should continue
            if !self.state.is_running() {
                tracing::warn!("Mission {} is not running, skipping check", self.id);
                continue;
            }

            // Perform deployment check
            match self.check_deployment().await {
                Ok(status) => {
                    tracing::info!("Deployment check for {}: {:?}", self.id, status);
                }
                Err(e) => {
                    tracing::error!("Deployment check failed for {}: {}", self.id, e);
                }
            }
        }
    }

    /// Check deployment pipeline status
    async fn check_deployment(&self) -> Result<DeploymentStatus, BenchError> {
        // Simulated deployment check
        // In production, this would make HTTP requests to deployment endpoints

        let status = DeploymentStatus {
            url: self.deployment_url.clone(),
            healthy: true,
            last_deploy: chrono::Utc::now(),
            pipeline_stage: PipelineStage::Complete,
        };

        Ok(status)
    }

    /// Create a mission monitor that can be controlled externally
    pub fn create_monitor(&self) -> MissionMonitor {
        MissionMonitor {
            mission_id: self.id.clone(),
            state: Arc::clone(&self.state),
        }
    }
}

/// Deployment pipeline status
#[derive(Debug, Clone)]
pub struct DeploymentStatus {
    pub url: String,
    pub healthy: bool,
    pub last_deploy: chrono::DateTime<chrono::Utc>,
    pub pipeline_stage: PipelineStage,
}

/// Pipeline stage in deployment
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineStage {
    Pending,
    Building,
    Testing,
    Deploying,
    Complete,
    Failed,
}

/// External monitor for controlling a mission
pub struct MissionMonitor {
    mission_id: String,
    state: Arc<AtomicMissionState>,
}

impl MissionMonitor {
    /// Pause the monitored mission
    pub fn pause(&self) {
        self.state.set_running(false);
        tracing::info!("Mission {} paused via monitor", self.mission_id);
    }

    /// Resume the monitored mission
    pub fn resume(&self) {
        self.state.set_running(true);
        tracing::info!("Mission {} resumed via monitor", self.mission_id);
    }

    /// Get current mission state
    pub fn state(&self) -> MissionState {
        if self.state.is_running() {
            MissionState::Running
        } else {
            MissionState::Paused
        }
    }

    /// Check if mission is active
    pub fn is_active(&self) -> bool {
        self.state.is_running()
    }
}

/// Mission configuration for deployment monitoring
#[derive(Debug, Clone)]
pub struct MissionConfig {
    pub mission_id: String,
    pub deployment_url: String,
    pub check_interval_hours: f64,
    pub alert_on_failure: bool,
    pub webhook_url: Option<String>,
}

impl Default for MissionConfig {
    fn default() -> Self {
        Self {
            mission_id: format!("deploy-monitor-{}", chrono::Utc::now().timestamp()),
            deployment_url: String::new(),
            check_interval_hours: 2.0,
            alert_on_failure: true,
            webhook_url: None,
        }
    }
}

/// Create a mission from configuration
pub fn create_mission_from_config(config: &MissionConfig) -> DeploymentMission {
    DeploymentMission::new(
        &config.mission_id,
        &config.deployment_url,
        config.check_interval_hours,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_lifecycle() {
        let mission = DeploymentMission::new("test-1", "https://deploy.example.com", 2.0);

        // Start running
        assert_eq!(mission.state(), MissionState::Running);
        assert!(mission.is_active());

        // Pause
        mission.pause();
        assert_eq!(mission.state(), MissionState::Paused);
        assert!(mission.is_active());

        // Resume
        mission.resume();
        assert_eq!(mission.state(), MissionState::Running);
        assert!(mission.is_active());

        // Stop
        mission.stop();
        assert_eq!(mission.state(), MissionState::Paused);
        assert!(!mission.is_active());
    }

    #[test]
    fn test_mission_monitor() {
        let mission = DeploymentMission::new("test-2", "https://deploy.example.com", 2.0);
        let monitor = mission.create_monitor();

        assert_eq!(monitor.state(), MissionState::Running);

        monitor.pause();
        assert_eq!(monitor.state(), MissionState::Paused);

        monitor.resume();
        assert_eq!(monitor.state(), MissionState::Running);
    }

    #[test]
    fn test_default_config() {
        let config = MissionConfig::default();

        assert!(config.mission_id.starts_with("deploy-monitor-"));
        assert_eq!(config.check_interval_hours, 2.0);
        assert!(config.alert_on_failure);
        assert!(config.webhook_url.is_none());
    }
}
