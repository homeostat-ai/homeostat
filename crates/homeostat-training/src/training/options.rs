use serde::{Deserialize, Serialize};

use crate::error::TrainingError;

pub const DEFAULT_HISTORY_WINDOW_MS: u64 = 24 * 60 * 60 * 1_000;
pub const DEFAULT_OUTCOME_HORIZON_MS: u64 = 15 * 60 * 1_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainingOptions {
    pub target_name: String,
    pub hidden_size: usize,
    pub bottleneck_size: usize,
    pub num_epochs: usize,
    pub batch_size: usize,
    pub num_workers: usize,
    pub seed: u64,
    pub learning_rate: f64,
    pub validation_fraction: f64,
    pub history_window_ms: u64,
    pub outcome_horizon_ms: u64,
    pub purge_gap_ms: u64,
}

impl Default for TrainingOptions {
    fn default() -> Self {
        Self {
            target_name: "max_node_cpu_ratio_after_15m".to_owned(),
            hidden_size: 64,
            bottleneck_size: 32,
            num_epochs: 50,
            batch_size: 32,
            num_workers: 0,
            seed: 42,
            learning_rate: 1.0e-3,
            validation_fraction: 0.2,
            history_window_ms: DEFAULT_HISTORY_WINDOW_MS,
            outcome_horizon_ms: DEFAULT_OUTCOME_HORIZON_MS,
            purge_gap_ms: DEFAULT_HISTORY_WINDOW_MS + DEFAULT_OUTCOME_HORIZON_MS,
        }
    }
}

impl TrainingOptions {
    pub(super) fn validate(&self) -> Result<(), TrainingError> {
        if self.target_name.trim().is_empty() {
            return Err(TrainingError::InvalidOptions(
                "target_name must not be empty".to_owned(),
            ));
        }
        if self.hidden_size == 0 || self.bottleneck_size == 0 {
            return Err(TrainingError::InvalidOptions(
                "hidden layer sizes must be greater than zero".to_owned(),
            ));
        }
        if self.num_epochs == 0 || self.batch_size == 0 {
            return Err(TrainingError::InvalidOptions(
                "num_epochs and batch_size must be greater than zero".to_owned(),
            ));
        }
        if !self.learning_rate.is_finite() || self.learning_rate <= 0.0 {
            return Err(TrainingError::InvalidOptions(
                "learning_rate must be finite and greater than zero".to_owned(),
            ));
        }
        if !(0.0..1.0).contains(&self.validation_fraction) || self.validation_fraction == 0.0 {
            return Err(TrainingError::InvalidOptions(
                "validation_fraction must be greater than zero and less than one".to_owned(),
            ));
        }

        let minimum_purge_gap = self
            .history_window_ms
            .saturating_add(self.outcome_horizon_ms);
        if self.purge_gap_ms < minimum_purge_gap {
            return Err(TrainingError::InvalidOptions(format!(
                "purge_gap_ms must be at least history_window_ms + outcome_horizon_ms ({minimum_purge_gap})"
            )));
        }
        Ok(())
    }
}
