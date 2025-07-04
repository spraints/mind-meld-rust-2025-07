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
        None | Some(Commands::Status) => cmd_status(config),
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

fn cmd_status(config: Config) {
    println!("{config:?}");
}
