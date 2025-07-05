mod app;
mod cli;
mod config;
mod dirs;
mod project;
mod store;

use std::collections::HashMap;
use std::rc::Rc;

use clap::Parser;
use config::Config;
use project::ProjectID;
use store::Store;

fn main() {
    let cli = cli::Cli::parse();
    let config = Config::load(cli.config.as_deref()).unwrap();
    match cli.command {
        None => cmd_status(Default::default(), config),
        Some(cli::Commands::Status(status_cmd)) => cmd_status(status_cmd, config),
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

fn cmd_status(cmd: cli::StatusCommand, config: Config) {
    let cli::StatusCommand { show_untracked } = cmd;
    let dirs = dirs::Dirs::new(&config).unwrap();

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

    if config.stores.is_empty() {
        println!("No stores yet!");
        println!("Get started by running '{} store create'.", exe());
        return;
    }

    println!("Stores:");
    let mut stores = Vec::new();
    let mut projects: HashMap<ProjectID, (bool, Vec<Rc<Store>>)> = HashMap::new();
    for st in &config.stores {
        match store::open(&st) {
            Ok(store) => {
                println!("  {st}");
                let store = Rc::new(store);
                stores.push(store.clone());
                match store.project_ids() {
                    Err(e) => println!("    error in repo: {e}"),
                    Ok(sp) => {
                        for proj in sp {
                            projects
                                .entry(proj)
                                .and_modify(|e| e.1.push(store.clone()))
                                .or_insert_with(|| (false, vec![store.clone()]));
                        }
                    }
                };
            }
            Err(e) => println!("  {st}: error: {e}"),
        };
    }
    println!();

    for proj in app::all_projects(dirs).expect("unexpected error") {
        projects
            .entry(proj)
            .and_modify(|e| e.0 = true)
            .or_insert((true, Vec::new()));
    }

    if projects.is_empty() {
        println!("No projects yet!");
        println!("Try creating a project in the Mindstorms or Spike Prime app next.");
        return;
    }

    println!(
        "todo: compare it all. Probably compare checksums? Or maybe it's easier just to walk through each part of the zip files."
    );
    println!();

    println!("Projects:");
    let mut untracked_count = 0;
    for (proj, (exists_locally, stores)) in projects {
        if stores.is_empty() && !show_untracked {
            untracked_count += 1;
            continue;
        }

        println!("  {proj}");
        if exists_locally {
            println!("    [CONTENT HASH] exists on disk");
        } else {
            println!("    (missing from disk)");
        }
        for st in stores {
            println!("    [CONTENT HASH] {st}");
        }
    }

    if untracked_count > 0 {
        println!(
            "  untracked: {untracked_count} (Run '{} status --untracked' to list them.)",
            exe()
        );
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
    for st in &config.stores {
        if store::paths_match(&st.path, &path) {
            println!("Already using {st}");
            return;
        }
    }
    let store = store::create(&store_type, path).unwrap().into();
    println!("Started using {}", store);
    config.stores.push(store);
    config.store().unwrap();
}

fn cmd_store_remove(args: cli::RemoveStoreArgs, mut config: Config) {
    let cli::RemoveStoreArgs { path } = args;
    let mut new_stores = Vec::new();
    let mut removed = 0;
    for st in config.stores {
        if store::paths_match(&st.path, &path) {
            println!("Stopped using {st}");
            removed += 1;
        } else {
            new_stores.push(st);
        }
    }
    config.stores = new_stores;
    config.store().unwrap();
    println!("Stores removed: {removed}");
}

fn exe() -> String {
    std::env::args()
        .next()
        .unwrap_or_else(|| "mind-meld".to_string())
}
