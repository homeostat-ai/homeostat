use std::{fs, path::Path};

use burn::{
    backend::Flex,
    prelude::{Config, Module},
    record::CompactRecorder,
};
use homeostat_model::{ArtifactManifest, OutcomeModelOptions};
use homeostat_training::{TrainingOptions, train};

#[test]
fn trains_and_exports_an_outcome_model() {
    let temporary = tempfile::tempdir().unwrap();
    let dataset_path = temporary.path().join("outcomes.csv");
    let artifact_dir = temporary.path().join("artifact");
    write_synthetic_dataset(&dataset_path);

    let options = TrainingOptions {
        hidden_size: 16,
        bottleneck_size: 8,
        num_epochs: 10,
        batch_size: 16,
        ..TrainingOptions::default()
    };
    let report = train(&dataset_path, &artifact_dir, options).unwrap();

    assert_eq!(report.training_examples, 72);
    assert_eq!(report.validation_examples, 24);
    assert_eq!(report.purged_examples, 24);
    assert!(report.validation_mae.is_finite());
    assert!(report.validation_rmse.is_finite());
    assert!(artifact_dir.join("model-options.json").is_file());
    assert!(artifact_dir.join("training-options.json").is_file());
    assert!(artifact_dir.join("normalization.json").is_file());
    assert!(artifact_dir.join("model.mpk").is_file());

    let device = Default::default();
    let model_options = OutcomeModelOptions::load(artifact_dir.join("model-options.json")).unwrap();
    model_options
        .init::<Flex>(&device)
        .load_file(artifact_dir.join("model"), &CompactRecorder::new(), &device)
        .unwrap();

    let manifest: ArtifactManifest =
        serde_json::from_slice(&fs::read(artifact_dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest.model_kind, "outcome-mlp-regressor");
    assert_eq!(manifest.training_examples, 72);
    assert_eq!(manifest.validation_examples, 24);
}

fn write_synthetic_dataset(path: &Path) {
    let mut writer = csv::Writer::from_path(path).unwrap();
    writer
        .write_record([
            "observed_at_unix_ms",
            "scenario_id",
            "target",
            "current_cpu_ratio",
            "cpu_mean_1h",
            "cpu_slope_6h",
            "action_move_shard",
            "action_horizontal_scale",
        ])
        .unwrap();

    for index in 0..120 {
        let current_cpu = 0.35 + (index % 20) as f32 * 0.02;
        let cpu_mean = current_cpu - 0.03;
        let cpu_slope = (index % 7) as f32 * 0.005 - 0.015;
        let move_shard = if index % 2 == 0 { 1.0 } else { 0.0 };
        let horizontal_scale = 1.0 - move_shard;
        let target = 0.15 + 0.55 * current_cpu + 0.2 * cpu_mean + 0.4 * cpu_slope
            - 0.08 * horizontal_scale
            - 0.03 * move_shard;
        let timestamp = 1_800_000_000_000_i64 + index * 60 * 60 * 1_000;

        writer
            .write_record([
                timestamp.to_string(),
                format!("scenario-{index:03}"),
                target.to_string(),
                current_cpu.to_string(),
                cpu_mean.to_string(),
                cpu_slope.to_string(),
                move_shard.to_string(),
                horizontal_scale.to_string(),
            ])
            .unwrap();
    }
    writer.flush().unwrap();
}
