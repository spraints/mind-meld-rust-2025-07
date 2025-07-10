use std::path::PathBuf;

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
    /// Commit changes
    Commit,
    /// Continuously add changes
    Watch,
}

#[derive(Args, Debug, Default)]
pub struct StatusCommand {
    /// Also show projects that aren't tracked yet.
    #[arg(long = "untracked")]
    pub show_untracked: bool,
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
