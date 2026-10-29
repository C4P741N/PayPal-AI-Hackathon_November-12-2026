use anyhow::{Context, Result};
use std::env;

fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let hello = env::var("ANTHROPIC_API_KEY").context("ANTHROPIC_API_KEY is missing from .env")?;

    println!("Printing {hello}");

    Ok(())
}
