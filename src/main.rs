mod app;
mod cli;
mod config;
mod dirs;
mod project;
mod status;
mod store;
mod track;
mod untrack;

use std::collections::{HashMap, HashSet};
use std::process::exit;
use std::rc::Rc;

use clap::Parser;
use config::Config;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use project::ProjectID;
use std::sync::mpsc::channel;
use std::time::Duration;
use store::Store;

fn main() {
    let cli = cli::Cli::parse();
    let config = Config::load(cli.config.as_deref()).unwrap();
    match cli.command {
        None => cmd_status(Default::default(), config),
        Some(cli::Commands::Status(status_cmd)) => cmd_status(status_cmd, config),
        Some(cli::Commands::Store(store_cmd)) => cmd_store(store_cmd, config),
        Some(cli::Commands::Track(track_cmd)) => cmd_track(track_cmd, config),
        Some(cli::Commands::Untrack(untrack_cmd)) => cmd_untrack(untrack_cmd, config),
        Some(cli::Commands::Commit) => cmd_commit(config),
        Some(cli::Commands::AutoCommit) => {
            cmd_auto_commit(config);
        }
    }
}

fn cmd_status(cmd: cli::StatusCommand, cfg: Config) {
    let cli::StatusCommand { show_untracked } = cmd;
    let dirs = dirs::Dirs::new(&cfg).unwrap();

    let mut any_overrides = false;
    if let Some(p) = &cfg.mindstorms_path {
        println!("Mindstorms path override: {p:?}");
        any_overrides = true;
    }
    if let Some(p) = &cfg.spike_path {
        println!("Spike path override: {p:?}");
        any_overrides = true;
    }
    if any_overrides {
        println!();
    }

    if cfg.stores.is_empty() {
        println!("No stores yet!");
        println!("Get started by running '{} store create'.", exe());
        return;
    }

    println!("Stores:");
    let mut all_stores = Vec::new();
    let mut projects: HashMap<ProjectID, Vec<Rc<Store>>> = HashMap::new();
    for st in &cfg.stores {
        match store::open(st) {
            Ok(store) => {
                println!("  {st}");
                let store = Rc::new(store);
                all_stores.push(store.clone());
                match store.project_ids() {
                    Err(e) => println!("    error in repo: {e}"),
                    Ok(sp) => {
                        for proj in sp {
                            projects
                                .entry(proj)
                                .and_modify(|e| e.push(store.clone()))
                                .or_insert_with(|| vec![store.clone()]);
                        }
                    }
                };
            }
            Err(e) => println!("  {st}: error: {e}"),
        };
    }
    println!();

    for proj in app::all_projects(&dirs).expect("unexpected error") {
        projects.entry(proj).or_default();
    }

    if projects.is_empty() {
        println!("No projects yet!");
        println!("Try creating a project in the Mindstorms or Spike Prime app next.");
        return;
    }

    println!("Projects:");
    let all_stores_count = all_stores.len();
    let mut untracked = Vec::new();
    for (proj, proj_stores) in projects {
        match proj_stores.is_empty() {
            true => untracked.push(proj),
            false => match status::get_status(&proj, &all_stores, &dirs) {
                Err(e) => println!("  {proj}! error: {e}"),
                Ok(status::Status::NoDifferences) => println!("  {proj}: up to date"),
                Ok(status::Status::LocalMissing) => {
                    println!("  {proj}: local copy has been deleted");
                    println!("    To stop tracking it, run:");
                    println!("      {} untrack --{} {:?}", exe(), proj.program, proj.name);
                }
                Ok(status::Status::Differences(out_of_date_stores)) => {
                    let store_list_count = out_of_date_stores.len();
                    let store_list: Vec<String> = out_of_date_stores
                        .iter()
                        .map(|st| format!("{st}"))
                        .collect();
                    let store_list = store_list.join("; ");
                    println!(
                        "  {proj}: {store_list_count}/{all_stores_count} stores need sync: {store_list}"
                    );
                }
            },
        };
    }

    if !untracked.is_empty() {
        println!();
        if show_untracked {
            for proj in untracked {
                println!("  (untracked) {proj}");
                println!(
                    "     track with: {} track --{} {:?}",
                    exe(),
                    proj.program,
                    proj.name
                );
            }
        } else {
            println!(
                "  untracked: {} (Run '{} status --untracked' to list them.)",
                untracked.len(),
                exe()
            );
        }
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

fn cmd_track(cmd: cli::TrackCommand, cfg: Config) {
    let cli::TrackCommand {
        spike,
        mindstorms,
        file_name,
    } = cmd;
    let res = match (spike, mindstorms) {
        (true, false) => track::track(cfg, project::Program::Spike, file_name),
        (false, true) => track::track(cfg, project::Program::Mindstorms, file_name),
        _ => {
            eprintln!("Exactly one of --spike or --mindstoms must be specified");
            exit(1);
        }
    };
    match res {
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
        Ok(res) => {
            let track::TrackResult { id, store_results } = res;
            println!("Now tracking {id}");
            let mut error_count = 0;
            for (st, st_res) in store_results {
                match st_res {
                    Ok(msg) => println!("  {st}: {msg}"),
                    Err(e) => {
                        error_count += 1;
                        println!("  {st}! error: {e}")
                    }
                };
            }
            if error_count > 0 {
                exit(1);
            }
        }
    };
}

fn cmd_untrack(cmd: cli::UntrackCommand, cfg: Config) {
    let cli::UntrackCommand {
        spike,
        mindstorms,
        file_name,
    } = cmd;
    let res = match (spike, mindstorms) {
        (true, false) => untrack::untrack(cfg, project::Program::Spike, file_name),
        (false, true) => untrack::untrack(cfg, project::Program::Mindstorms, file_name),
        _ => {
            eprintln!("Exactly one of --spike or --mindstorms must be specified");
            exit(1);
        }
    };
    match res {
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
        Ok(res) => {
            let untrack::UntrackResult { id, store_results } = res;
            println!("Stopped tracking {id}");
            let mut error_count = 0;
            for (st, st_res) in store_results {
                match st_res {
                    Ok(msg) => println!("  {st}: {msg}"),
                    Err(e) => {
                        error_count += 1;
                        println!("  {st}! error: {e}")
                    }
                };
            }
            if error_count > 0 {
                exit(1);
            }
        }
    };
}

fn cmd_commit(cfg: Config) {
    let dirs = dirs::Dirs::new(&cfg).unwrap();

    if cfg.stores.is_empty() {
        println!("No stores yet!");
        println!("Get started by running '{} store create'.", exe());
        return;
    }

    // Find all tracked projects
    let mut tracked_projects = HashSet::new();
    for st in &cfg.stores {
        match store::open(st) {
            Ok(store) => {
                match store.project_ids() {
                    Err(e) => println!("Error reading store {st}: {e}"),
                    Ok(project_ids) => {
                        for proj_id in project_ids {
                            tracked_projects.insert(proj_id);
                        }
                    }
                };
            }
            Err(e) => println!("Error opening store {st}: {e}"),
        };
    }

    if tracked_projects.is_empty() {
        println!("No tracked projects found!");
        println!("Track a project first with '{} track'.", exe());
        return;
    }

    // Read all tracked projects
    let mut projects_to_commit = Vec::new();
    for proj_id in &tracked_projects {
        match project::read(proj_id, &dirs) {
            Ok(Some(raw_project)) => {
                projects_to_commit.push((proj_id, raw_project));
            }
            Ok(None) => {
                println!("Project {proj_id} does not exist.");
                println!("  To stop tracking it, run:");
                println!(
                    "    {} untrack --{} {:?}",
                    exe(),
                    proj_id.program,
                    proj_id.name
                );
                println!();
            }
            Err(e) => {
                println!("Error reading project {proj_id}: {e}");
            }
        }
    }

    if projects_to_commit.is_empty() {
        println!("No projects could be read!");
        return;
    }

    // Commit to all stores
    println!(
        "Committing {} projects to {} stores...",
        projects_to_commit.len(),
        cfg.stores.len()
    );

    let mut error_count = 0;
    for st in &cfg.stores {
        match store::open(st) {
            Ok(store) => {
                let project_refs: Vec<(&project::ProjectID, &project::RawProject)> =
                    projects_to_commit
                        .iter()
                        .map(|(id, proj)| (*id, proj))
                        .collect();

                match store.commit(&project_refs, "Update tracked projects") {
                    Ok(msg) => println!("  {st}: {msg}"),
                    Err(e) => {
                        error_count += 1;
                        println!("  {st}! error: {e}")
                    }
                };
            }
            Err(e) => {
                error_count += 1;
                println!("  {st}! error opening store: {e}")
            }
        };
    }

    if error_count > 0 {
        println!("Completed with {} errors", error_count);
        exit(1);
    } else {
        println!("Successfully committed all projects to all stores");
    }
}

fn cmd_auto_commit(cfg: Config) {
    let dirs = dirs::Dirs::new(&cfg).unwrap();

    if cfg.stores.is_empty() {
        println!("No stores yet!");
        println!("Get started by running '{} store create'.", exe());
        return;
    }

    // Find all tracked projects
    let mut tracked_projects = HashSet::new();
    for st in &cfg.stores {
        match store::open(st) {
            Ok(store) => {
                match store.project_ids() {
                    Err(e) => println!("Error reading store {st}: {e}"),
                    Ok(project_ids) => {
                        for proj_id in project_ids {
                            tracked_projects.insert(proj_id);
                        }
                    }
                };
            }
            Err(e) => println!("Error opening store {st}: {e}"),
        };
    }

    if tracked_projects.is_empty() {
        println!("No tracked projects found!");
        println!("Track a project first with '{} track'.", exe());
        return;
    }

    // Build a map of project id to file path
    let mut project_paths = Vec::new();
    for proj_id in &tracked_projects {
        let base_path = match proj_id.program {
            project::Program::Mindstorms => &dirs.mindstorms,
            project::Program::Spike => &dirs.spike,
        };
        let path = base_path.join(&proj_id.name);
        project_paths.push((proj_id.clone(), path));
    }

    // TODO - handle ctrl-C for graceful shutdown.

    // Set up file watcher
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Default::default()).unwrap();
    for (proj_id, path) in &project_paths {
        if let Err(e) = watcher.watch(path, RecursiveMode::NonRecursive) {
            println!("Failed to watch {:?}: {e}", path);
        }
    }
    println!("Watching for changes to tracked files...");

    for res in rx {
        match res {
            Ok(event) => {
                let path = &event.paths[0]; // todo handle all paths.

                // Find which project this path matches
                // todo emit a warning for paths that we don't recognize.
                if let Some((proj_id, _)) = project_paths.iter().find(|(_, p)| p == path) {
                    println!("got change to {proj_id}");
                    // Read the project
                    match project::read(proj_id, &dirs) {
                        Ok(Some(raw_project)) => {
                            // Commit to all stores
                            let mut error_count = 0;
                            for st in &cfg.stores {
                                match store::open(st) {
                                    Ok(store) => {
                                        let project_refs = vec![(*proj_id, &raw_project)];
                                        match store.commit(
                                            &project_refs,
                                            "Update tracked projects via auto-commit",
                                        ) {
                                            Ok(_) => println!("updated {}", path.display()),
                                            Err(e) => {
                                                error_count += 1;
                                                println!("  {st}! error: {e}")
                                            }
                                        };
                                    }
                                    Err(e) => {
                                        error_count += 1;
                                        println!("  {st}! error opening store: {e}")
                                    }
                                };
                            }
                            if error_count > 0 {
                                println!("Completed with {} errors", error_count);
                            }
                        }
                        Ok(None) => {
                            println!("Project {proj_id} does not exist.");
                        }
                        Err(e) => {
                            println!("Error reading project {proj_id}: {e}");
                        }
                    }
                }
            }
            Err(e) => println!("watch error: {e}"),
        }
    }
}

fn exe() -> String {
    std::env::args()
        .next()
        .unwrap_or_else(|| "mind-meld".to_string())
}
