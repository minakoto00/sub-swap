mod cli;
mod config;
mod crypto;
mod error;
mod guard;
mod paths;
mod profile;
mod tui;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sub-swap", version, about = "Manage multiple ~/.codex/ profiles")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all profiles
    List {
        #[arg(short, long)]
        verbose: bool,
    },
    /// Switch to a profile
    Use {
        name: String,
        #[arg(short, long)]
        force: bool,
    },
    /// Import current ~/.codex/ config as a new profile
    Add {
        name: String,
        #[arg(long)]
        from: Option<String>,
        #[arg(short, long)]
        note: Option<String>,
    },
    /// Delete a stored profile
    Remove { name: String },
    /// Rename a profile
    Rename { old: String, new: String },
    /// Set or update a profile's note
    Note { name: String, text: String },
    /// View decrypted profile contents (stdout only)
    Decrypt { name: String },
    /// Manage global settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a config value
    Set { key: String, value: String },
    /// Show current config
    Show,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = cli::run(cli) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
