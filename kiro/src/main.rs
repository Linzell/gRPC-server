// main.rs
#![recursion_limit = "256"]
#![allow(clippy::enum_variant_names)]

#[macro_use]
extern crate lazy_static;
#[cfg(feature = "postgres")]
extern crate diesel;
#[cfg(feature = "postgres")]
extern crate diesel_migrations;
#[cfg(feature = "surrealdb")]
extern crate surrealdb;
#[cfg(feature = "surrealdb")]
extern crate surrealdb_migrations;

use std::path::Path;

#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};

#[cfg(feature = "cli")]
mod cmd;
mod config;
mod prelude;
mod utils;

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "DIR")]
    config: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    Configfile {},
    // CreateApiKey {
    //     #[arg(short, long, value_name = "NAME")]
    //     name: String,
    // },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "cli")]
    let cli = Cli::parse();

    #[cfg(feature = "cli")]
    config::load(Path::new(&cli.config))?;

    #[cfg(not(feature = "cli"))]
    config::load(Path::new("config.toml"))?;

    #[cfg(feature = "tracing")]
    let conf = config::get();

    #[cfg(feature = "tracing")]
    utils::telemetry::init_tracer(&conf)?;

    #[cfg(feature = "cli")]
    match &cli.command {
        Some(Commands::Configfile {}) => cmd::configfile::run(),
        // Some(Commands::CreateApiKey { name }) => cmd::create_api_key::run(name).await?,
        None => cmd::root::run().await?,
    }
    #[cfg(not(feature = "cli"))]
    cmd::root::run().await?;

    Ok(())
}
