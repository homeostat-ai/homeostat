#[cfg(feature = "training")]
use std::path::PathBuf;

#[cfg(feature = "training")]
use clap::Args;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "homeostat",
    version,
    about = "A learning-augmented control plane for sharded systems"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Operate the Homeostat controller.
    Controller {
        #[command(subcommand)]
        command: ControllerCommand,
    },
    #[cfg(feature = "training")]
    /// Work with Homeostat models.
    Model {
        #[command(subcommand)]
        command: ModelCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum ControllerCommand {
    /// Start the controller process.
    Start,
}

#[cfg(feature = "training")]
#[derive(Debug, Subcommand)]
pub enum ModelCommand {
    /// Train an action-outcome model.
    Train(ModelTrainArgs),
}

#[cfg(feature = "training")]
#[derive(Debug, Args)]
pub struct ModelTrainArgs {
    /// CSV dataset produced by the offline dataset builder.
    #[arg(long)]
    pub dataset: PathBuf,

    /// Empty or non-existent directory where the model artifact will be written.
    #[arg(long)]
    pub artifact_dir: PathBuf,

    #[arg(long, default_value = "max_node_cpu_ratio_after_15m")]
    pub target_name: String,

    #[arg(long, default_value_t = 50)]
    pub epochs: usize,

    #[arg(long, default_value_t = 32)]
    pub batch_size: usize,

    #[arg(long, default_value_t = 64)]
    pub hidden_size: usize,

    #[arg(long, default_value_t = 32)]
    pub bottleneck_size: usize,

    #[arg(long, default_value_t = 1.0e-3)]
    pub learning_rate: f64,

    #[arg(long, default_value_t = 0.2)]
    pub validation_fraction: f64,

    #[arg(long, default_value_t = 24)]
    pub history_hours: u64,

    #[arg(long, default_value_t = 15)]
    pub outcome_horizon_minutes: u64,

    /// Gap between training and validation. It must be at least the history
    /// window plus the outcome horizon.
    #[arg(long, default_value_t = 1_455)]
    pub purge_gap_minutes: u64,

    #[arg(long, default_value_t = 42)]
    pub seed: u64,

    #[arg(long, default_value_t = 0)]
    pub num_workers: usize,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[cfg(feature = "training")]
    use super::ModelCommand;
    use super::{Cli, Command, ControllerCommand};

    #[test]
    fn parses_controller_start() {
        let cli = Cli::try_parse_from(["homeostat", "controller", "start"]).unwrap();

        assert!(matches!(
            cli.command,
            Command::Controller {
                command: ControllerCommand::Start
            }
        ));
    }

    #[test]
    fn requires_a_controller_action() {
        assert!(Cli::try_parse_from(["homeostat", "controller"]).is_err());
    }

    #[cfg(feature = "training")]
    #[test]
    fn parses_model_training() {
        let cli = Cli::try_parse_from([
            "homeostat",
            "model",
            "train",
            "--dataset",
            "samples.csv",
            "--artifact-dir",
            "artifacts/model-v1",
        ])
        .unwrap();

        let Command::Model {
            command: ModelCommand::Train(args),
        } = cli.command
        else {
            panic!("expected model train command");
        };
        assert_eq!(args.epochs, 50);
        assert_eq!(args.history_hours, 24);
        assert_eq!(args.outcome_horizon_minutes, 15);
    }
}
