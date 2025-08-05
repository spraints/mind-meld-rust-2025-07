use std::path::PathBuf;
use std::time::Duration;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "mm")]
#[command(about = "Mind meld CLI", long_about = None)]
pub struct Cli {
    /// Config file location (default is system-dependent).
    #[arg(long, global = true)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show status (this is the default action)
    Status(StatusCommand),
    /// Manage stores (e.g., git repos)
    Store(StoreCommand),
    /// Track a file
    Track(TrackCommand),
    /// Untrack a file
    Untrack(UntrackCommand),
    /// Commit changes
    Commit,
    /// Automatically commit changes as they happen
    AutoCommit(AutoCommitCommand),
    /// Show commit history
    Log(LogCommand),
    /// Render tracked projects to a directory
    Render(RenderCommand),
    /*
     * todo: render to file
     * todo: render in GUI (?)
     * todo: render in browser (?)
     * todo: render to branch (can do automatically)
     * todo: render diff (given a commit id and store)
     * todo: revert to a specific version
     * todo: accept python branch
     */
}

#[derive(Args, Debug, Default)]
pub struct StatusCommand {
    /// Also show projects that aren't tracked yet.
    #[arg(long = "untracked")]
    pub show_untracked: bool,
}

#[derive(Args, Debug)]
pub struct LogCommand {
    /// Duration to look back (e.g., "1d", "2w", "1h")
    #[arg(long, default_value = "1d", value_parser = parse_duration)]
    pub since: Duration,

    /// Store to show logs from (if not specified, uses the only store if there's just one)
    #[arg(long)]
    pub store: Option<PathBuf>,
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    if s.is_empty() {
        return Err("duration cannot be empty".to_string());
    }

    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: u64 = num_str
        .parse()
        .map_err(|_| format!("invalid number: {}", num_str))?;

    match unit {
        "s" => Ok(Duration::from_secs(num)),
        "m" => Ok(Duration::from_secs(num * 60)),
        "h" => Ok(Duration::from_secs(num * 60 * 60)),
        "d" => Ok(Duration::from_secs(num * 24 * 60 * 60)),
        "w" => Ok(Duration::from_secs(num * 7 * 24 * 60 * 60)),
        _ => Err(format!("invalid duration unit: {}", unit)),
    }
}

#[derive(Args, Debug)]
pub struct StoreCommand {
    #[command(subcommand)]
    pub subcommand: StoreSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum StoreSubcommand {
    /// Create a store (e.g., a git repo)
    Create(CreateStoreArgs),
    /// Remove a store
    Remove(RemoveStoreArgs),
}

#[derive(Args, Debug)]
pub struct CreateStoreArgs {
    /// The store type (git is the only option)
    #[arg(long = "type")]
    pub store_type: String,
    /// Path to the repo
    pub path: PathBuf,
}

#[derive(Args, Debug)]
pub struct RemoveStoreArgs {
    /// Path to the repo
    pub path: PathBuf,
}

#[derive(Args, Debug)]
pub struct TrackCommand {
    /// Track a spike file
    #[arg(long)]
    pub spike: bool,
    /// Track a mindstorms file
    #[arg(long)]
    pub mindstorms: bool,

    pub file_name: PathBuf,
}

#[derive(Args, Debug)]
pub struct UntrackCommand {
    /// Untrack a spike file
    #[arg(long)]
    pub spike: bool,
    /// Untrack a mindstorms file
    #[arg(long)]
    pub mindstorms: bool,

    pub file_name: PathBuf,
}

#[derive(Args, Debug)]
pub struct AutoCommitCommand {
    /// Minimum interval between commits
    #[arg(long, default_value = "30s", value_parser = parse_duration)]
    pub interval: Duration,
}

#[derive(Args, Debug)]
pub struct RenderCommand {
    #[arg(short, long)]
    pub out_dir: PathBuf,

    /// Which store to pull data from (must be specified if there's more than one store configured)
    #[arg(long)]
    pub store: Option<PathBuf>,

    // Which revision to render (default is the most recent commit)
    #[arg(long)]
    pub revision: Option<String>,
}
