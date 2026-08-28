//! Privileged validation and execution of system-specific shard operations.

use async_trait::async_trait;
use homeostat_api::v1::Action;

/// Stable custom OpenTelemetry attributes emitted by executor implementations.
/// Dataset builders join action spans to observations through these values.
pub mod trace_attributes {
    pub const SYSTEM_ID: &str = "homeostat.system.id";
    pub const ACTION_ID: &str = "homeostat.action.id";
    pub const ACTION_KIND: &str = "homeostat.action.kind";
    pub const BEFORE_OBSERVATION_ID: &str = "homeostat.observation.before.id";
    pub const EXECUTION_STATUS: &str = "homeostat.execution.status";
}

/// Current progress of an in-process execution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OperationPhase {
    #[default]
    Unspecified,
    Validating,
    Preparing,
    Transferring,
    CatchingUp,
    Activating,
    Draining,
    Stabilizing,
    Completed,
    Failed,
    Cancelled,
}

/// Lightweight execution state for callers in the same process.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Operation {
    pub operation_id: String,
    pub action_id: String,
    pub phase: OperationPhase,
    pub message: String,
    pub started_at_unix_ms: Option<i64>,
    pub updated_at_unix_ms: Option<i64>,
}

/// The result of validating an action against current system state and safety rules.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActionValidation {
    pub violations: Vec<String>,
}

impl ActionValidation {
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Validates and executes actions while preserving system-specific correctness.
///
/// Implementations emit one trace span per action with the stable attributes
/// above. Action results remain observability data rather than protobuf API
/// contracts; the training pipeline derives outcomes by joining those spans to
/// immutable observations.
#[async_trait]
pub trait Executor: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn validate(&self, action: &Action) -> Result<ActionValidation, Self::Error>;

    async fn execute(&self, action: Action, dry_run: bool) -> Result<Operation, Self::Error>;

    async fn operation(&self, operation_id: &str) -> Result<Operation, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::ActionValidation;

    #[test]
    fn validation_is_valid_when_there_are_no_violations() {
        assert!(ActionValidation::default().is_valid());
    }

    #[test]
    fn validation_is_invalid_when_a_violation_is_present() {
        let validation = ActionValidation {
            violations: vec!["snapshot revision changed".to_owned()],
        };

        assert!(!validation.is_valid());
    }
}
