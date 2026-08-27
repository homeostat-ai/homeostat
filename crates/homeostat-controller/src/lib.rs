//! Planning, policy evaluation, and the Homeostat control loop.

use async_trait::async_trait;
use homeostat_api::v1::{ClusterSnapshot, Plan};

/// Proposes a plan from a canonical cluster snapshot.
#[async_trait]
pub trait Policy: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn propose(&self, snapshot: &ClusterSnapshot) -> Result<Plan, Self::Error>;
}

/// A deterministic policy that allows the observation pipeline to run safely before balancing
/// policies are implemented.
#[derive(Debug, Default)]
pub struct NoOpPolicy;

#[async_trait]
impl Policy for NoOpPolicy {
    type Error = std::convert::Infallible;

    async fn propose(&self, snapshot: &ClusterSnapshot) -> Result<Plan, Self::Error> {
        Ok(Plan {
            plan_id: String::new(),
            system_id: snapshot.system_id.clone(),
            expected_revision: snapshot.revision,
            actions: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{NoOpPolicy, Policy};
    use homeostat_api::v1::ClusterSnapshot;

    #[test]
    fn no_op_policy_preserves_snapshot_revision() {
        let snapshot = ClusterSnapshot {
            system_id: "test-system".to_owned(),
            revision: 42,
            ..Default::default()
        };

        let plan = futures_executor::block_on(NoOpPolicy.propose(&snapshot)).unwrap();

        assert_eq!(plan.system_id, snapshot.system_id);
        assert_eq!(plan.expected_revision, snapshot.revision);
        assert!(plan.actions.is_empty());
    }
}
