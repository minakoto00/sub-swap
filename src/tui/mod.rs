pub mod wizard;
pub mod widgets;

use crate::error::Result;
use crate::paths::Paths;

pub fn run_tui(_paths: &Paths) -> Result<()> {
    println!("Interactive TUI — coming soon. Use `sub-swap --help` for CLI commands.");
    Ok(())
}
