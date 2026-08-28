use crate::error::DatasetError;

use super::{DatasetSplit, OutcomeDataset};

impl OutcomeDataset {
    /// Splits samples chronologically and drops training rows near the
    /// validation boundary. The purge gap prevents overlapping history and
    /// outcome windows from leaking into validation.
    pub fn chronological_split(
        &self,
        validation_fraction: f64,
        purge_gap_ms: u64,
    ) -> Result<DatasetSplit, DatasetError> {
        if !(0.0..1.0).contains(&validation_fraction) || validation_fraction == 0.0 {
            return Err(DatasetError::InvalidValidationFraction(validation_fraction));
        }

        let validation_len = ((self.samples.len() as f64) * validation_fraction)
            .ceil()
            .clamp(1.0, (self.samples.len() - 1) as f64) as usize;
        let validation_start_index = self.samples.len() - validation_len;
        let validation_start_ms = self.samples[validation_start_index].observed_at_unix_ms;
        let purge_gap_ms = i64::try_from(purge_gap_ms).unwrap_or(i64::MAX);
        let latest_training_ms = validation_start_ms.saturating_sub(purge_gap_ms);

        let training = self.samples[..validation_start_index]
            .iter()
            .filter(|sample| sample.observed_at_unix_ms <= latest_training_ms)
            .cloned()
            .collect::<Vec<_>>();
        let purged_examples = validation_start_index - training.len();
        let validation = self.samples[validation_start_index..].to_vec();

        if training.len() < 2 || validation.is_empty() {
            return Err(DatasetError::SplitTooSmall {
                training: training.len(),
                validation: validation.len(),
                purge_gap_ms,
            });
        }

        Ok(DatasetSplit {
            training,
            validation,
            purged_examples,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_dataset;

    #[test]
    fn chronological_split_purges_rows_near_validation() {
        let split = test_dataset().chronological_split(0.2, 2_000).unwrap();

        assert_eq!(split.training.len(), 7);
        assert_eq!(split.purged_examples, 1);
        assert_eq!(split.validation.len(), 2);
        assert_eq!(split.validation[0].observed_at_unix_ms, 8_000);
    }
}
