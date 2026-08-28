use burn::{backend::Flex, prelude::Tensor, tensor::backend::BackendTypes};
use homeostat_model::{Normalization, OutcomeModel};

use crate::{dataset::OutcomeSample, error::TrainingError};

pub(super) fn evaluate(
    model: &OutcomeModel<Flex>,
    normalization: &Normalization,
    samples: &[OutcomeSample],
    batch_size: usize,
    device: &<Flex as BackendTypes>::Device,
) -> Result<(f32, f32), TrainingError> {
    let mut absolute_error = 0.0_f64;
    let mut squared_error = 0.0_f64;

    for batch in samples.chunks(batch_size) {
        let mut features = Vec::with_capacity(batch.len() * normalization.feature_names.len());
        for sample in batch {
            features.extend(normalization.normalize_features(&sample.features)?);
        }
        let features = Tensor::<Flex, 1>::from_floats(features.as_slice(), device)
            .reshape([batch.len(), normalization.feature_names.len()]);
        let predictions = model
            .forward(features)
            .into_data()
            .to_vec::<f32>()
            .map_err(|error| TrainingError::Burn(error.to_string()))?;

        for (prediction, sample) in predictions.into_iter().zip(batch) {
            let prediction = normalization.denormalize_target(prediction);
            let error = f64::from(prediction - sample.target);
            absolute_error += error.abs();
            squared_error += error * error;
        }
    }

    let count = samples.len() as f64;
    let mae = (absolute_error / count) as f32;
    let rmse = (squared_error / count).sqrt() as f32;
    if !mae.is_finite() || !rmse.is_finite() {
        return Err(TrainingError::NonFiniteMetrics { mae, rmse });
    }
    Ok((mae, rmse))
}
