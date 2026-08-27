//! Privileged validation and execution of system-specific shard operations.

use async_trait::async_trait;
use homeostat_api::v1::{Operation, Plan, ValidatePlanResponse};

/// Validates and executes plans while preserving system-specific correctness.
#[async_trait]
pub trait Executor: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn validate(&self, plan: &Plan) -> Result<ValidatePlanResponse, Self::Error>;

    async fn execute(
        &self,
        plan: Plan,
        idempotency_key: &str,
        dry_run: bool,
    ) -> Result<Operation, Self::Error>;

    async fn operation(&self, operation_id: &str) -> Result<Operation, Self::Error>;
}
