use clap::Parser;

use args::Args;

mod args;

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => eprintln!("Error: {e}"),
    }
}

fn run() -> anyhow::Result<(), String> {
    let args = Args::parse();
    if args.generate_completions().is_ok() {
        return Ok(());
    }

    #[cfg(feature = "gui")]
    if args.is_call_gui() {
        return gui::main().map_err(|e| e.to_string());
    }

    return cli::main(args.cli()).map_err(|e| e.to_string());
}
