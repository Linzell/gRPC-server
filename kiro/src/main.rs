// main.rs
#![recursion_limit = "256"]
#![allow(clippy::enum_variant_names)]

#[macro_use]
extern crate lazy_static;
#[cfg(feature = "postgres")]
extern crate diesel_migrations;
#[cfg(feature = "surrealdb")]
extern crate surrealdb_migrations;

#[macro_use]
#[cfg(feature = "postgres")]
extern crate diesel;
#[cfg(feature = "surrealdb")]
extern crate surrealdb;

use std::path::Path;

use clap::{Parser, Subcommand};

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

    CreateApiKey {
        #[arg(short, long, value_name = "NAME")]
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "cli")]
    let cli = Cli::parse();

    #[cfg(feature = "cli")]
    config::load(Path::new(&cli.config))?;

    let conf = config::get();

    #[cfg(feature = "tracing")]
    utils::telemetry::init_tracer(&conf);

    Ok(())
}
