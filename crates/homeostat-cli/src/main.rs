mod banner;
mod cli;
mod controller;
mod observability;

use clap::Parser;
use cli::{Cli, Command, ControllerCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Controller {
            command: ControllerCommand::Start,
        } => controller::start().await,
    }
}
