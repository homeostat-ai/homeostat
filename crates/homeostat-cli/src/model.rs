use homeostat_training::{TrainingOptions, train as train_model};

use crate::cli::ModelTrainArgs;

const MILLIS_PER_MINUTE: u64 = 60 * 1_000;
const MILLIS_PER_HOUR: u64 = 60 * MILLIS_PER_MINUTE;

pub fn train(args: ModelTrainArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let options = TrainingOptions {
        target_name: args.target_name,
        hidden_size: args.hidden_size,
        bottleneck_size: args.bottleneck_size,
        num_epochs: args.epochs,
        batch_size: args.batch_size,
        num_workers: args.num_workers,
        seed: args.seed,
        learning_rate: args.learning_rate,
        validation_fraction: args.validation_fraction,
        history_window_ms: args.history_hours.saturating_mul(MILLIS_PER_HOUR),
        outcome_horizon_ms: args
            .outcome_horizon_minutes
            .saturating_mul(MILLIS_PER_MINUTE),
        purge_gap_ms: args.purge_gap_minutes.saturating_mul(MILLIS_PER_MINUTE),
    };

    let report = train_model(args.dataset, args.artifact_dir, options)?;
    println!("training complete");
    println!("  artifacts   {}", report.artifact_dir.display());
    println!("  train rows  {}", report.training_examples);
    println!("  valid rows  {}", report.validation_examples);
    println!("  purged rows {}", report.purged_examples);
    println!("  valid MAE   {:.6}", report.validation_mae);
    println!("  valid RMSE  {:.6}", report.validation_rmse);
    Ok(())
}
