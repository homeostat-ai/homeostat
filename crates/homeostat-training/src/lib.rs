//! Offline dataset preparation, training, evaluation, and artifact export.

pub mod dataset;
pub mod error;
pub mod training;

pub use error::{DatasetError, TrainingError};
pub use training::{TrainingOptions, TrainingReport, train};
