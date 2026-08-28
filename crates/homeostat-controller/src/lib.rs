//! Planning, policy evaluation, and the Homeostat control loop.

use async_trait::async_trait;
use homeostat_api::v1::{Action, NoOp, Observation, action};

/// Proposes one action from a canonical system observation.
#[async_trait]
pub trait Policy: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn propose(&self, observation: &Observation) -> Result<Action, Self::Error>;
}

/// A deterministic policy that allows the observation pipeline to run safely before balancing
/// policies are implemented.
#[derive(Debug, Default)]
pub struct NoOpPolicy;

#[async_trait]
impl Policy for NoOpPolicy {
    type Error = std::convert::Infallible;

    async fn propose(&self, observation: &Observation) -> Result<Action, Self::Error> {
        Ok(Action {
            action_id: format!(
                "noop:{}:{}",
                observation.observation_id, observation.revision
            ),
            expected_revision: Some(observation.revision),
            kind: Some(action::Kind::NoOp(NoOp {})),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{NoOpPolicy, Policy};
    use homeostat_api::v1::Observation;

    #[test]
    fn no_op_policy_preserves_observation_revision() {
        let observation = Observation {
            system_id: "test-system".to_owned(),
            revision: 42,
            ..Default::default()
        };

        let action = futures_executor::block_on(NoOpPolicy.propose(&observation)).unwrap();

        assert_eq!(action.expected_revision, Some(observation.revision));
        assert!(matches!(
            action.kind,
            Some(homeostat_api::v1::action::Kind::NoOp(_))
        ));
    }
}
