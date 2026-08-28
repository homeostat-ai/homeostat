mod csv;
mod normalization;
mod split;
mod types;

pub use normalization::fit_normalization;
pub use types::{DatasetSplit, OutcomeDataset, OutcomeSample};

#[cfg(test)]
fn test_dataset() -> OutcomeDataset {
    OutcomeDataset {
        feature_names: vec!["cpu".to_owned(), "constant".to_owned()],
        samples: (0..10)
            .map(|index| OutcomeSample {
                observed_at_unix_ms: index * 1_000,
                scenario_id: "scenario-a".to_owned(),
                features: vec![index as f32, 1.0],
                target: (index * 2) as f32,
            })
            .collect(),
    }
}
