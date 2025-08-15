mod app;
mod cli;
mod commit;
mod config;
mod dirs;
mod project;
mod render;
mod status;
mod store;
mod track;
mod untrack;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::exit;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use clap::Parser;
use config::{Config, StoreConfig};
use notify_debouncer_full::notify::{Error, RecursiveMode};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, DebouncedEvent};
use project::ProjectID;
use std::sync::mpsc::channel;
use store::{Revision, Store};

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
        Some(cli::Commands::AutoCommit(auto_commit_cmd)) => {
            cmd_auto_commit(auto_commit_cmd, config);
        }
        Some(cli::Commands::Log(log_cmd)) => cmd_log(log_cmd, config),
        Some(cli::Commands::Render(render_cmd)) => cmd_render(render_cmd, config),
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
    let (all_stores, err_stores) = store::open_all(&cfg.stores);
    for (st, e) in err_stores {
        println!("  {st}: error opening store: {e}");
    }
    let all_stores: Vec<Rc<Store>> = all_stores.into_iter().map(|(_, s)| Rc::new(s)).collect();
    let mut projects: HashMap<ProjectID, Vec<Rc<Store>>> = HashMap::new();
    for store in &all_stores {
        println!("  {store}");
        match store.project_ids() {
            Err(e) => println!("    could not get tracked projects: {e}"),
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
    let (stores, err_stores) = store::open_all(&cfg.stores);
    for (st, e) in err_stores {
        println!("{st}: error opening store: {e}");
    }
    let (tracked_projects, errs) = store::all_project_ids(&stores);
    for (st, e) in errs {
        println!("{st}: error reading projects: {e}");
    }

    if tracked_projects.is_empty() {
        println!("No tracked projects found!");
        println!("Track a project first with '{} track'.", exe());
        return;
    }

    // Commit to all stores
    println!(
        "Committing {} projects to {} stores...",
        tracked_projects.len(),
        cfg.stores.len()
    );

    let (stores, err_stores) = store::open_all(&cfg.stores);
    for (st, e) in err_stores {
        println!("  {st}! error opening store: {e}")
    }

    let commit::CommitResult {
        missing_projects,
        project_read_errors,
        store_results,
    } = commit::commit(&stores, &dirs, &tracked_projects, "Update tracked projects");

    for proj_id in missing_projects {
        println!("Project {proj_id} does not exist on this computer.");
        println!("  To stop tracking it, run:");
        println!(
            "    {} untrack --{} {:?}",
            exe(),
            proj_id.program,
            proj_id.name
        );
        println!();
    }
    for (proj_id, e) in project_read_errors {
        println!("{proj_id}: error reading project: {e}");
    }
    for (st, res) in store_results {
        match res {
            Ok(msg) => println!("{st}: {msg}"),
            Err(e) => println!("{st}! {e}"),
        };
    }
}

fn cmd_auto_commit(opts: cli::AutoCommitCommand, cfg: Config) {
    let cli::AutoCommitCommand { interval } = opts;
    let dirs = dirs::Dirs::new(&cfg).unwrap();

    if cfg.stores.is_empty() {
        println!("No stores yet!");
        println!("Get started by running '{} store create'.", exe());
        return;
    }

    // Find all tracked projects
    let (stores, err_stores) = store::open_all(&cfg.stores);
    for (st, e) in err_stores {
        println!("{st}: error opening store: {e}");
    }
    let (tracked_projects, errs) = store::all_project_ids(&stores);
    for (st, e) in errs {
        println!("{st}: error reading projects: {e}");
    }

    if tracked_projects.is_empty() {
        println!("No tracked projects found!");
        println!("Track a project first with '{} track'.", exe());
        return;
    }

    // Build a map of project id to file path
    let mut project_paths = HashMap::new();
    for proj_id in &tracked_projects {
        let path = proj_id.path(&dirs);
        project_paths.insert(path, proj_id);
    }

    // Set up file watcher
    enum AutoCommitEvent {
        DebouncedEvent(Vec<DebouncedEvent>),
        WatchError(Error),
        ControlC,
    }
    let (tx, rx) = channel();

    {
        let tx = tx.clone();
        let _ = ctrlc::set_handler(move || {
            let _ = tx.send(AutoCommitEvent::ControlC);
        });
    }

    let mut debouncer = {
        let tx = tx.clone();
        new_debouncer(
            interval,
            Some(Duration::from_millis(500)),
            move |res: DebounceEventResult| match res {
                Ok(events) => {
                    let _ = tx.send(AutoCommitEvent::DebouncedEvent(events));
                }
                Err(errs) => {
                    for err in errs {
                        let _ = tx.send(AutoCommitEvent::WatchError(err));
                    }
                }
            },
        )
        .unwrap()
    };

    let mut any_watches = false;
    for path in project_paths.keys() {
        match debouncer.watch(path, RecursiveMode::NonRecursive) {
            Err(e) => println!("{path:?}: failed to watch: {e}"),
            Ok(_) => any_watches = true,
        };
    }
    if !any_watches {
        exit(1);
    }

    println!("Watching for changes to tracked files...");

    for res in rx {
        match res {
            AutoCommitEvent::DebouncedEvent(events) => {
                println!(
                    "[{}] Auto-committing changed projects:",
                    chrono::Local::now()
                );

                let mut proj_ids = HashSet::new();
                for e in events {
                    for p in &e.paths {
                        match project_paths.get(p) {
                            Some(x) => {
                                println!("{x}: {:?}", e.kind);
                                proj_ids.insert((*x).clone());
                            }
                            None => {
                                println!("unexpected {:?}: {:?}", e.kind, p);
                            }
                        };
                    }
                }

                let (stores, err_stores) = store::open_all(&cfg.stores);
                for (st, e) in err_stores {
                    println!("{st}! error opening store: {e}")
                }

                do_auto_commit(&stores, &dirs, &proj_ids);
                println!();
            }
            AutoCommitEvent::WatchError(e) => println!("watch error: {e}"),
            AutoCommitEvent::ControlC => {
                println!(
                    "[{}] Auto-committing changed projects on shutdown:",
                    chrono::Local::now()
                );
                do_auto_commit(&stores, &dirs, &tracked_projects);
                return;
            }
        }
    }
}

fn do_auto_commit(
    stores: &[(StoreConfig, Store)],
    dirs: &dirs::Dirs,
    proj_ids: &HashSet<ProjectID>,
) {
    let commit::CommitResult {
        missing_projects,
        project_read_errors,
        store_results,
    } = commit::commit(
        stores,
        dirs,
        proj_ids,
        "Update tracked projects via auto-commit",
    );

    for proj_id in missing_projects {
        println!("Project {proj_id} is now missing.");
        println!("  To stop tracking it, run:");
        println!(
            "    {} untrack --{} {:?}",
            exe(),
            proj_id.program,
            proj_id.name
        );
    }
    for (proj_id, e) in project_read_errors {
        println!("{proj_id}: error reading project: {e}");
    }
    for (st, res) in store_results {
        match res {
            Ok(msg) => println!("{st}: {msg}"),
            Err(e) => println!("{st}! {e}"),
        };
    }
}

fn cmd_log(cmd: cli::LogCommand, cfg: Config) {
    let cli::LogCommand { since, store } = cmd;

    let target_store = match get_single_store(&cfg, store) {
        None => exit(1),
        Some(s) => s,
    };

    // Open the store
    let store = match store::open(target_store) {
        Ok(s) => s,
        Err(e) => {
            println!("  {target_store}: error opening store: {e}");
            exit(1);
        }
    };

    match store.log(SystemTime::now() - since) {
        Err(e) => {
            eprintln!("Failed to get log: {}", e);
            exit(1);
        }
        Ok(store::LogResult::Unborn) => println!("No commits in this store."),
        Ok(store::LogResult::None(newest_commit)) => println!(
            "No commits in the last {}, newest commit is {} old.",
            format_duration_ago(since),
            format_time_ago(newest_commit.date)
        ),
        Ok(store::LogResult::Some(commits)) => {
            for commit in commits {
                println!(
                    "{} {} ({}) {}",
                    commit.hash,
                    format_time_ago(commit.date),
                    format_datetime(commit.date),
                    commit.message
                );
                for proj_id in commit.changed_projects {
                    println!("  +/- {proj_id}");
                }
            }
        }
    };
}

fn cmd_render(opts: cli::RenderCommand, cfg: config::Config) {
    let cli::RenderCommand {
        dest,
        store,
        revision,
    } = opts;
    let cli::RenderDest {
        out_dir,
        to_store: tree,
    } = dest;

    let target_store = match get_single_store(&cfg, store) {
        None => exit(1),
        Some(s) => s,
    };
    let store = match store::open(target_store) {
        Ok(s) => s,
        Err(e) => {
            println!("{target_store}: error opening store: {e}");
            exit(1);
        }
    };
    let revision = match revision {
        None => Revision::Latest,
        Some(ref expr) => match store.resolve(expr) {
            Ok(r) => r,
            Err(e) => {
                println!("{target_store}: error resolving {expr}: {e}");
                exit(1);
            }
        },
    };

    let res = match (out_dir, tree) {
        (Some(out_dir), false) => render::render_all_projects(
            render::fs::out_dir(out_dir),
            render::txt::TextFormatter,
            &store,
            revision,
        ),
        (None, true) => render::render_all_projects(
            render::store::tree(&store),
            render::txt::TextFormatter,
            &store,
            revision,
        ),
        _ => unreachable!(),
    };

    match res {
        Ok(_) => (),
        Err(e) => println!("error: {e}"),
    };
}

fn get_single_store(cfg: &config::Config, store: Option<PathBuf>) -> Option<&StoreConfig> {
    if cfg.stores.is_empty() {
        println!("No stores yet!");
        println!("Get started by running '{} store create'.", exe());
        return None;
    }

    // Determine which store to use
    match store {
        Some(store_path) => {
            // Find the store that matches the provided path
            match std::path::absolute(store_path) {
                Err(_) => {
                    eprintln!("Invalid store path");
                    None
                }
                Ok(store_path) => {
                    let matching_store = cfg.stores.iter().find(|s| s.path == store_path);
                    match matching_store {
                        Some(s) => Some(s),
                        None => {
                            eprintln!("Store not found: {}", store_path.display());
                            None
                        }
                    }
                }
            }
        }
        None => {
            // Use the only store if there's just one
            if cfg.stores.len() == 1 {
                Some(&cfg.stores[0])
            } else {
                eprintln!("Multiple stores available. Please specify one with --store:");
                for store in &cfg.stores {
                    println!("  {}", store.relpath().display());
                }
                None
            }
        }
    }
}

const SECONDS_IN_MINUTE: u64 = 60;
const SECONDS_IN_HOUR: u64 = 60 * SECONDS_IN_MINUTE;
const SECONDS_IN_DAY: u64 = 24 * SECONDS_IN_HOUR;
const SECONDS_IN_WEEK: u64 = 7 * SECONDS_IN_DAY;

fn format_time_ago(time: SystemTime) -> String {
    match SystemTime::now().duration_since(time) {
        Ok(d) => format_duration_ago(d),
        Err(e) => format!("(?? {e} ??)"),
    }
}

fn format_duration_ago(d: Duration) -> String {
    match d {
        d if d >= Duration::from_secs(SECONDS_IN_WEEK) => {
            format!("{}w", d.as_secs() / SECONDS_IN_WEEK)
        }
        d if d >= Duration::from_secs(SECONDS_IN_DAY) => {
            format!("{}d", d.as_secs() / SECONDS_IN_DAY)
        }
        d if d >= Duration::from_secs(SECONDS_IN_HOUR) => {
            format!("{}h", d.as_secs() / SECONDS_IN_HOUR)
        }
        d if d >= Duration::from_secs(SECONDS_IN_MINUTE) => {
            format!("{}m", d.as_secs() / SECONDS_IN_MINUTE)
        }
        d => format!("{d:?}"),
    }
}

fn format_datetime(time: SystemTime) -> String {
    use chrono::{DateTime, Local};
    let datetime: DateTime<Local> = DateTime::from(time);
    datetime.format("%Y-%m-%d %H:%M").to_string()
}

fn exe() -> String {
    std::env::args()
        .next()
        .unwrap_or_else(|| "mind-meld".to_string())
}
