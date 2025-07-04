use clap::{Parser, Subcommand, Args};

#[derive(Parser, Debug)]
#[command(name = "mm")]
#[command(about = "Mind meld CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage stores (e.g., git repos)
    Store(StoreCommand),
    /// Track a file
    Track(TrackCommand),
    /// Snapshot changes
    Snapshot,
    /// Continuously add changes
    Watch,
}

#[derive(Args, Debug)]
pub struct StoreCommand {
    #[command(subcommand)]
    pub subcommand: StoreSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum StoreSubcommand {
    /// Create a store (e.g., a git repo)
    Create {
        /// Use git as the store backend
        #[arg(long)]
        git: bool,
        /// Path to the repo
        path: String,
    },
    /// Remove a store
    Remove {
        /// Path to the repo
        path: String,
    },
}

#[derive(Args, Debug)]
pub struct TrackCommand {
    /// Track a spike file
    #[arg(long)]
    pub spike: Option<String>,
    /// Track a mindstorms file
    #[arg(long)]
    pub mindstorms: Option<String>,
} 