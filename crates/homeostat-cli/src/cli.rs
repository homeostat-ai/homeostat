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
}

#[derive(Debug, Subcommand)]
pub enum ControllerCommand {
    /// Start the controller process.
    Start,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

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
}
