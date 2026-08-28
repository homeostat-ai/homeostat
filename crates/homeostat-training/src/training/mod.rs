mod artifact;
mod batch;
mod evaluation;
mod options;
mod runner;

pub use options::{DEFAULT_HISTORY_WINDOW_MS, DEFAULT_OUTCOME_HORIZON_MS, TrainingOptions};
pub use runner::{TrainingReport, train};
