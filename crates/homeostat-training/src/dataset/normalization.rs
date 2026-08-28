use homeostat_model::Normalization;

use crate::error::DatasetError;

use super::OutcomeSample;

const MIN_STANDARD_DEVIATION: f64 = 1.0e-6;

pub fn fit_normalization(
    feature_names: &[String],
    samples: &[OutcomeSample],
) -> Result<Normalization, DatasetError> {
    let first = samples.first().ok_or(DatasetError::NotEnoughSamples(0))?;
    let width = feature_names.len();
    if first.features.len() != width || samples.iter().any(|sample| sample.features.len() != width)
    {
        return Err(DatasetError::InconsistentFeatureWidth);
    }

    let count = samples.len() as f64;
    let mut feature_means = vec![0.0_f64; width];
    let mut target_mean = 0.0_f64;
    for sample in samples {
        for (mean, value) in feature_means.iter_mut().zip(&sample.features) {
            *mean += f64::from(*value);
        }
        target_mean += f64::from(sample.target);
    }
    for mean in &mut feature_means {
        *mean /= count;
    }
    target_mean /= count;

    let mut feature_variances = vec![0.0_f64; width];
    let mut target_variance = 0.0_f64;
    for sample in samples {
        for ((variance, value), mean) in feature_variances
            .iter_mut()
            .zip(&sample.features)
            .zip(&feature_means)
        {
            let difference = f64::from(*value) - mean;
            *variance += difference * difference;
        }
        let difference = f64::from(sample.target) - target_mean;
        target_variance += difference * difference;
    }

    let feature_stds = feature_variances
        .into_iter()
        .map(|variance| stable_std(variance / count) as f32)
        .collect();
    let target_std = stable_std(target_variance / count) as f32;

    Ok(Normalization {
        feature_names: feature_names.to_vec(),
        feature_means: feature_means
            .into_iter()
            .map(|value| value as f32)
            .collect(),
        feature_stds,
        target_mean: target_mean as f32,
        target_std,
    })
}

fn stable_std(variance: f64) -> f64 {
    let std = variance.sqrt();
    if std < MIN_STANDARD_DEVIATION {
        1.0
    } else {
        std
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_dataset, fit_normalization};

    #[test]
    fn normalization_uses_training_statistics_and_handles_constants() {
        let dataset = test_dataset();
        let normalization = fit_normalization(&dataset.feature_names, &dataset.samples).unwrap();

        assert_eq!(normalization.feature_means, vec![4.5, 1.0]);
        assert_eq!(normalization.feature_stds[1], 1.0);
        assert_eq!(normalization.target_mean, 9.0);
    }
}
