//! Read-only collection and normalization of sharded-system state.

use async_trait::async_trait;
use homeostat_api::v1::ClusterSnapshot;

/// Produces canonical snapshots without requiring system write access.
#[async_trait]
pub trait StateSource: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn snapshot(&self) -> Result<ClusterSnapshot, Self::Error>;
}
