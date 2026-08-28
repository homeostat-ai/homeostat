use std::path::{Path, PathBuf};

use burn::{
    backend::{Autodiff, Flex},
    data::{dataloader::DataLoaderBuilder, dataset::InMemDataset},
    optim::AdamConfig,
    prelude::{Backend, Config, Module},
    record::CompactRecorder,
    train::{Learner, SupervisedTraining, metric::LossMetric},
};
use homeostat_model::{
    ARTIFACT_VERSION, ArtifactManifest, FEATURE_SCHEMA_VERSION, OutcomeBatch, OutcomeModelOptions,
};

use crate::{
    dataset::{OutcomeDataset, OutcomeSample, fit_normalization},
    error::TrainingError,
};

use super::{
    artifact::{prepare_artifact_dir, write_json},
    batch::OutcomeBatcher,
    evaluation::evaluate,
    options::TrainingOptions,
};

type TrainingBackend = Autodiff<Flex>;
type InferenceBackend = Flex;

#[derive(Clone, Debug, PartialEq)]
pub struct TrainingReport {
    pub artifact_dir: PathBuf,
    pub training_examples: usize,
    pub validation_examples: usize,
    pub purged_examples: usize,
    pub validation_mae: f32,
    pub validation_rmse: f32,
}

pub fn train(
    dataset_path: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    options: TrainingOptions,
) -> Result<TrainingReport, TrainingError> {
    options.validate()?;
    let dataset = OutcomeDataset::from_csv(dataset_path)?;
    let split = dataset.chronological_split(options.validation_fraction, options.purge_gap_ms)?;
    let normalization = fit_normalization(&dataset.feature_names, &split.training)?;
    normalization.validate()?;

    let artifact_dir = artifact_dir.as_ref();
    prepare_artifact_dir(artifact_dir)?;

    let model_options = OutcomeModelOptions::new(dataset.feature_names.len())
        .with_hidden_size(options.hidden_size)
        .with_bottleneck_size(options.bottleneck_size);
    model_options
        .save(artifact_dir.join("model-options.json"))
        .map_err(|error| TrainingError::Burn(error.to_string()))?;
    write_json(artifact_dir.join("training-options.json"), &options)?;
    write_json(artifact_dir.join("normalization.json"), &normalization)?;

    let device = Default::default();
    TrainingBackend::seed(&device, options.seed);
    let batcher = OutcomeBatcher::new(normalization.clone())?;

    let dataloader_train =
        DataLoaderBuilder::<TrainingBackend, OutcomeSample, OutcomeBatch<TrainingBackend>>::new(
            batcher.clone(),
        )
        .batch_size(options.batch_size)
        .shuffle(options.seed)
        .num_workers(options.num_workers)
        .set_device(device)
        .build(InMemDataset::new(split.training.clone()));

    let dataloader_validation =
        DataLoaderBuilder::<InferenceBackend, OutcomeSample, OutcomeBatch<InferenceBackend>>::new(
            batcher,
        )
        .batch_size(options.batch_size)
        .num_workers(options.num_workers)
        .set_device(device)
        .build(InMemDataset::new(split.validation.clone()));

    let training = SupervisedTraining::new(artifact_dir, dataloader_train, dataloader_validation)
        .metrics((LossMetric::new(),))
        .with_file_checkpointer(CompactRecorder::new())
        .num_epochs(options.num_epochs)
        .summary();
    let model = model_options.init::<TrainingBackend>(&device);
    let result = training.launch(Learner::new(
        model,
        AdamConfig::new().init(),
        options.learning_rate,
    ));

    let (validation_mae, validation_rmse) = evaluate(
        &result.model,
        &normalization,
        &split.validation,
        options.batch_size,
        &device,
    )?;

    result
        .model
        .save_file(artifact_dir.join("model"), &CompactRecorder::new())
        .map_err(|error| TrainingError::Burn(error.to_string()))?;

    let manifest = ArtifactManifest {
        artifact_version: ARTIFACT_VERSION,
        feature_schema_version: FEATURE_SCHEMA_VERSION,
        model_kind: "outcome-mlp-regressor".to_owned(),
        target_name: options.target_name.clone(),
        history_window_ms: options.history_window_ms,
        outcome_horizon_ms: options.outcome_horizon_ms,
        training_examples: split.training.len(),
        validation_examples: split.validation.len(),
        validation_mae,
        validation_rmse,
    };
    write_json(artifact_dir.join("manifest.json"), &manifest)?;

    Ok(TrainingReport {
        artifact_dir: artifact_dir.to_path_buf(),
        training_examples: split.training.len(),
        validation_examples: split.validation.len(),
        purged_examples: split.purged_examples,
        validation_mae,
        validation_rmse,
    })
}
