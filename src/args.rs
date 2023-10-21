use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, shells, Generator};

#[derive(Debug, Parser)]
pub(crate) struct Args {
    #[command(flatten)]
    cli: cli::Cli,

    #[command(subcommand)]
    command: Option<ArgsCommand>,
}

impl Args {
    pub(crate) fn cli(&self) -> &cli::Cli {
        &self.cli
    }
    pub(crate) fn generate_completions(&self) -> Result<(), ()> {
        let Some(ArgsCommand::Completions { sh }) = &self.command else {
            return Err(());
        };
        match sh {
            ShellVariant::Bash => generate_completions(shells::Bash),
            ShellVariant::Fish => generate_completions(shells::Fish),
            ShellVariant::Zsh => generate_completions(shells::Zsh),
        };
        Ok(())
    }
    #[cfg(feature = "gui")]
    pub(crate) fn is_call_gui(&self) -> bool {
        matches!(self.command, Some(ArgsCommand::Gui))
    }
}

#[derive(Debug, Subcommand)]
enum ArgsCommand {
    /// Generate completions
    Completions {
        #[clap(name = "shell")]
        /// For which shell to generate completions
        sh: ShellVariant,
    },
    #[cfg(feature = "gui")]
    /// Open gui
    Gui,
}

#[derive(Debug, Clone, ValueEnum)]
enum ShellVariant {
    Bash,
    Fish,
    Zsh,
}

fn generate_completions<G: Generator>(sh: G) {
    let mut command = Args::command();
    generate(
        sh,
        &mut command,
        env!("CARGO_BIN_NAME"),
        &mut std::io::stdout(),
    )
}
