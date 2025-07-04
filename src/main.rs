mod cli;
mod config;
mod dirs;
use clap::Parser;
use cli::{Cli, Commands, StoreSubcommand};
use config::Config;

fn main() {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref()).unwrap();
    match &cli.command {
        None | Some(Commands::Status) => cmd_status(&config),
        Some(Commands::Store(store_cmd)) => match &store_cmd.subcommand {
            StoreSubcommand::Create { git, path } => {
                println!("Create store: git={}, path={}", git, path);
            }
            StoreSubcommand::Remove { path } => {
                println!("Remove store: path={}", path);
            }
        },
        Some(Commands::Track(track_cmd)) => {
            if let Some(spike) = &track_cmd.spike {
                println!("Track spike file: {}", spike);
            }
            if let Some(mindstorms) = &track_cmd.mindstorms {
                println!("Track mindstorms file: {}", mindstorms);
            }
            if track_cmd.spike.is_none() && track_cmd.mindstorms.is_none() {
                println!("No file specified to track.");
            }
        }
        Some(Commands::Snapshot) => {
            println!("Snapshot changes (not yet implemented)");
        }
        Some(Commands::Watch) => {
            println!("Watch for changes (not yet implemented)");
        }
    }
}

fn cmd_status(config: &Config) {
    let mut any_overrides = false;
    if let Some(p) = &config.mindstorms_path {
        println!("Mindstorms path override: {p:?}");
        any_overrides = true;
    }
    if let Some(p) = &config.spike_path {
        println!("Spike path override: {p:?}");
        any_overrides = true;
    }
    if any_overrides {
        println!();
    }

    if config.stores.len() == 0 {
        println!("Get started by running '{} store create'.", exe());
    }
    for store in &config.stores {
        println!("todo: show store state: {store:?}");
        // I think I'm going to keep track of which files are
    }
}

fn exe() -> String {
    std::env::args()
        .next()
        .unwrap_or_else(|| "mind-meld".to_string())
}
