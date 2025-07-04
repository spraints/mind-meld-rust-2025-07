mod cli;
mod config;
mod dirs;
mod store;

use clap::Parser;
use config::Config;

fn main() {
    let cli = cli::Cli::parse();
    let config = Config::load(cli.config.as_deref()).unwrap();
    match cli.command {
        None | Some(cli::Commands::Status) => cmd_status(config),
        Some(cli::Commands::Store(store_cmd)) => cmd_store(store_cmd, config),
        Some(cli::Commands::Track(track_cmd)) => {
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
        Some(cli::Commands::Snapshot) => {
            println!("Snapshot changes (not yet implemented)");
        }
        Some(cli::Commands::Watch) => {
            println!("Watch for changes (not yet implemented)");
        }
    }
}

fn cmd_status(config: Config) {
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
        println!("No stores yet!");
        println!("Get started by running '{} store create'.", exe());
    }
    for store in &config.stores {
        println!("todo: show store state: {store:?}");
        // I think I'm going to keep track of which files are
    }
}

fn cmd_store(cmd: cli::StoreCommand, config: Config) {
    match cmd.subcommand {
        cli::StoreSubcommand::Create(args) => cmd_store_create(args, config),
        cli::StoreSubcommand::Remove(args) => cmd_store_remove(args, config),
    }
}

fn cmd_store_create(args: cli::CreateStoreArgs, mut config: Config) {
    let cli::CreateStoreArgs { store_type, path } = args;
    let store = store::create(&store_type, path).unwrap();
    config.stores.push(store.into());
    config.store().unwrap();
}

fn cmd_store_remove(args: cli::RemoveStoreArgs, config: Config) {
    println!("todo: remove store {args:?}");
}

fn exe() -> String {
    std::env::args()
        .next()
        .unwrap_or_else(|| "mind-meld".to_string())
}
