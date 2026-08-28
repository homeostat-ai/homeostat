use std::path::PathBuf;

use homeostat_model::NormalizationError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatasetError {
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error("dataset is missing required column `{0}`")]
    MissingColumn(&'static str),
    #[error("dataset must contain at least one numeric feature column")]
    NoFeatures,
    #[error("row {row} has an empty scenario_id")]
    EmptyScenario { row: usize },
    #[error("row {row} has invalid value `{value}` in column `{column}`")]
    InvalidField {
        row: usize,
        column: String,
        value: String,
    },
    #[error("row {row} has a non-finite value in column `{column}`")]
    NonFiniteField { row: usize, column: String },
    #[error("dataset contains {0} samples; at least 3 are required")]
    NotEnoughSamples(usize),
    #[error("validation_fraction must be greater than 0 and less than 1, got {0}")]
    InvalidValidationFraction(f64),
    #[error(
        "time split is too small after purging: training={training}, validation={validation}, purge_gap_ms={purge_gap_ms}"
    )]
    SplitTooSmall {
        training: usize,
        validation: usize,
        purge_gap_ms: i64,
    },
    #[error("samples do not all match the declared feature width")]
    InconsistentFeatureWidth,
}

#[derive(Debug, Error)]
pub enum TrainingError {
    #[error(transparent)]
    Dataset(#[from] DatasetError),
    #[error(transparent)]
    Normalization(#[from] NormalizationError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("invalid training options: {0}")]
    InvalidOptions(String),
    #[error("artifact path is not a directory: {0}")]
    ArtifactPathIsNotDirectory(PathBuf),
    #[error("artifact directory must be empty: {0}")]
    ArtifactDirectoryNotEmpty(PathBuf),
    #[error("Burn operation failed: {0}")]
    Burn(String),
    #[error("validation produced non-finite metrics: mae={mae}, rmse={rmse}")]
    NonFiniteMetrics { mae: f32, rmse: f32 },
}
