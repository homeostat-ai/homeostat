//! Shared feature, artifact, and inference contracts for Homeostat models.

use burn::{
    nn::{Linear, LinearConfig, Relu},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Version of the fixed-width feature-vector contract consumed by the model.
pub const FEATURE_SCHEMA_VERSION: u32 = 1;

/// Version of the on-disk artifact manifest.
pub const ARTIFACT_VERSION: u32 = 1;

/// A small multilayer perceptron that predicts the normalized outcome of an
/// action from a fixed-width system-and-action feature vector.
#[derive(Module, Debug)]
pub struct OutcomeModel<B: Backend> {
    input: Linear<B>,
    hidden: Linear<B>,
    output: Linear<B>,
    activation: Relu,
}

#[derive(Config, Debug)]
pub struct OutcomeModelOptions {
    pub input_size: usize,
    #[config(default = 64)]
    pub hidden_size: usize,
    #[config(default = 32)]
    pub bottleneck_size: usize,
}

impl OutcomeModelOptions {
    pub fn init<B: Backend>(&self, device: &B::Device) -> OutcomeModel<B> {
        OutcomeModel {
            input: LinearConfig::new(self.input_size, self.hidden_size).init(device),
            hidden: LinearConfig::new(self.hidden_size, self.bottleneck_size).init(device),
            output: LinearConfig::new(self.bottleneck_size, 1).init(device),
            activation: Relu::new(),
        }
    }
}

impl<B: Backend> OutcomeModel<B> {
    /// Runs the outcome regressor.
    ///
    /// Shapes:
    /// - `features`: `[batch_size, input_size]`
    /// - return value: `[batch_size, 1]`
    pub fn forward(&self, features: Tensor<B, 2>) -> Tensor<B, 2> {
        let features = self.input.forward(features);
        let features = self.activation.forward(features);
        let features = self.hidden.forward(features);
        let features = self.activation.forward(features);
        self.output.forward(features)
    }
}

/// A normalized batch shared by the offline trainer and the model's training
/// step.
#[derive(Clone, Debug)]
pub struct OutcomeBatch<B: Backend> {
    pub features: Tensor<B, 2>,
    pub targets: Tensor<B, 2>,
}

#[cfg(feature = "training")]
mod training {
    use burn::{
        nn::loss::{MseLoss, Reduction},
        tensor::backend::AutodiffBackend,
        train::{InferenceStep, RegressionOutput, TrainOutput, TrainStep},
    };

    use super::{OutcomeBatch, OutcomeModel};

    impl<B: burn::prelude::Backend> OutcomeModel<B> {
        fn forward_regression(&self, batch: OutcomeBatch<B>) -> RegressionOutput<B> {
            let output = self.forward(batch.features);
            let loss =
                MseLoss::new().forward(output.clone(), batch.targets.clone(), Reduction::Mean);
            RegressionOutput::new(loss, output, batch.targets)
        }
    }

    impl<B: AutodiffBackend> TrainStep for OutcomeModel<B> {
        type Input = OutcomeBatch<B>;
        type Output = RegressionOutput<B>;

        fn step(&self, batch: Self::Input) -> TrainOutput<Self::Output> {
            let item = self.forward_regression(batch);
            TrainOutput::new(self, item.loss.backward(), item)
        }
    }

    impl<B: burn::prelude::Backend> InferenceStep for OutcomeModel<B> {
        type Input = OutcomeBatch<B>;
        type Output = RegressionOutput<B>;

        fn step(&self, batch: Self::Input) -> Self::Output {
            self.forward_regression(batch)
        }
    }
}

/// Statistics fitted on the training split and reused unchanged for validation
/// and inference.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Normalization {
    pub feature_names: Vec<String>,
    pub feature_means: Vec<f32>,
    pub feature_stds: Vec<f32>,
    pub target_mean: f32,
    pub target_std: f32,
}

impl Normalization {
    pub fn normalize_features(&self, values: &[f32]) -> Result<Vec<f32>, NormalizationError> {
        self.validate_feature_width(values.len())?;

        Ok(values
            .iter()
            .zip(&self.feature_means)
            .zip(&self.feature_stds)
            .map(|((value, mean), std)| (value - mean) / std)
            .collect())
    }

    pub fn normalize_target(&self, value: f32) -> f32 {
        (value - self.target_mean) / self.target_std
    }

    pub fn denormalize_target(&self, value: f32) -> f32 {
        value * self.target_std + self.target_mean
    }

    pub fn validate(&self) -> Result<(), NormalizationError> {
        if self.feature_names.len() != self.feature_means.len()
            || self.feature_names.len() != self.feature_stds.len()
        {
            return Err(NormalizationError::InconsistentFeatureWidths {
                names: self.feature_names.len(),
                means: self.feature_means.len(),
                stds: self.feature_stds.len(),
            });
        }

        if self.target_std <= 0.0 || !self.target_std.is_finite() {
            return Err(NormalizationError::InvalidTargetStandardDeviation(
                self.target_std,
            ));
        }

        if let Some((index, value)) = self
            .feature_stds
            .iter()
            .enumerate()
            .find(|(_, value)| **value <= 0.0 || !value.is_finite())
        {
            return Err(NormalizationError::InvalidFeatureStandardDeviation {
                index,
                value: *value,
            });
        }

        Ok(())
    }

    fn validate_feature_width(&self, actual: usize) -> Result<(), NormalizationError> {
        self.validate()?;
        let expected = self.feature_names.len();
        if actual != expected {
            return Err(NormalizationError::FeatureWidth { expected, actual });
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum NormalizationError {
    #[error("feature vector has width {actual}, expected {expected}")]
    FeatureWidth { expected: usize, actual: usize },
    #[error(
        "normalization arrays have inconsistent widths: names={names}, means={means}, stds={stds}"
    )]
    InconsistentFeatureWidths {
        names: usize,
        means: usize,
        stds: usize,
    },
    #[error("feature {index} has invalid standard deviation {value}")]
    InvalidFeatureStandardDeviation { index: usize, value: f32 },
    #[error("target has invalid standard deviation {0}")]
    InvalidTargetStandardDeviation(f32),
}

/// Metadata required to interpret and evaluate a saved model.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ArtifactManifest {
    pub artifact_version: u32,
    pub feature_schema_version: u32,
    pub model_kind: String,
    pub target_name: String,
    pub history_window_ms: u64,
    pub outcome_horizon_ms: u64,
    pub training_examples: usize,
    pub validation_examples: usize,
    pub validation_mae: f32,
    pub validation_rmse: f32,
}

#[cfg(test)]
mod tests {
    use burn::{backend::Flex, prelude::Tensor};

    use super::{Normalization, NormalizationError, OutcomeModelOptions};

    #[test]
    fn model_returns_one_outcome_per_sample() {
        let device = Default::default();
        let model = OutcomeModelOptions::new(3).init::<Flex>(&device);
        let input = Tensor::<Flex, 2>::from_floats([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], &device);

        assert_eq!(model.forward(input).dims(), [2, 1]);
    }

    #[test]
    fn normalization_checks_feature_width() {
        let normalization = Normalization {
            feature_names: vec!["cpu".to_owned(), "memory".to_owned()],
            feature_means: vec![0.5, 0.5],
            feature_stds: vec![0.25, 0.25],
            target_mean: 0.5,
            target_std: 0.25,
        };

        assert_eq!(
            normalization.normalize_features(&[0.5]),
            Err(NormalizationError::FeatureWidth {
                expected: 2,
                actual: 1,
            })
        );
    }
}
