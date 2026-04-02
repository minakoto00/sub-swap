use crate::error::Result;
use crate::paths::Paths;

pub fn run_first_launch(_paths: &Paths) -> Result<()> {
    println!("First launch wizard — coming soon.");
    Ok(())
}
