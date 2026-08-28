use burn::{
    data::dataloader::batcher::Batcher,
    prelude::{Backend, Tensor},
};
use homeostat_model::{Normalization, NormalizationError, OutcomeBatch};

use crate::dataset::OutcomeSample;

#[derive(Clone)]
pub(super) struct OutcomeBatcher {
    normalization: Normalization,
}

impl OutcomeBatcher {
    pub(super) fn new(normalization: Normalization) -> Result<Self, NormalizationError> {
        normalization.validate()?;
        Ok(Self { normalization })
    }
}

impl<B: Backend> Batcher<B, OutcomeSample, OutcomeBatch<B>> for OutcomeBatcher {
    fn batch(&self, items: Vec<OutcomeSample>, device: &B::Device) -> OutcomeBatch<B> {
        let batch_size = items.len();
        let input_size = self.normalization.feature_names.len();
        let mut features = Vec::with_capacity(batch_size * input_size);
        let mut targets = Vec::with_capacity(batch_size);

        for item in items {
            debug_assert_eq!(item.features.len(), input_size);
            features.extend(
                item.features
                    .iter()
                    .zip(&self.normalization.feature_means)
                    .zip(&self.normalization.feature_stds)
                    .map(|((value, mean), std)| (value - mean) / std),
            );
            targets.push(self.normalization.normalize_target(item.target));
        }

        let features = Tensor::<B, 1>::from_floats(features.as_slice(), device)
            .reshape([batch_size, input_size]);
        let targets =
            Tensor::<B, 1>::from_floats(targets.as_slice(), device).reshape([batch_size, 1]);

        OutcomeBatch { features, targets }
    }
}
