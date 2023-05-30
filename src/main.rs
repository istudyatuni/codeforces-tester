#[cfg(any(feature = "gui", feature = "cli"))]
use clap::Parser;

#[cfg(any(feature = "gui", feature = "cli"))]
use args::Args;

mod args;

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => eprintln!("Error: {e}"),
    }
}

fn run() -> anyhow::Result<(), String> {
    #[cfg(any(feature = "gui", feature = "cli"))]
    let args = Args::parse();

    #[cfg(feature = "gui")]
    if args.gui {
        return gui::main().map_err(|e| e.to_string());
    }

    #[cfg(feature = "cli")]
    return cli::main(args.cli()).map_err(|e| e.to_string());

    #[cfg(not(any(feature = "gui", feature = "cli")))]
    Err(
        "nothing compiled, just chilling. try to use --features or --all-features when building"
            .to_string(),
    )
}
