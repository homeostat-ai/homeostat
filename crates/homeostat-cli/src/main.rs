mod banner;
mod cli;
mod controller;
#[cfg(feature = "training")]
mod model;
mod observability;

use clap::Parser;
#[cfg(feature = "training")]
use cli::ModelCommand;
use cli::{Cli, Command, ControllerCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Controller {
            command: ControllerCommand::Start,
        } => controller::start().await,
        #[cfg(feature = "training")]
        Command::Model {
            command: ModelCommand::Train(args),
        } => model::train(args),
    }
}
