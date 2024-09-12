// main.rs
#![recursion_limit = "256"]

#[macro_use]
extern crate lazy_static;
#[cfg(feature = "postgres")]
extern crate diesel_migrations;

#[macro_use]
#[cfg(feature = "postgres")]
extern crate diesel;

use std::path::Path;

use clap::{Parser, Subcommand};

mod config;
mod error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "DIR")]
    config: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

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
    let cli = Cli::parse();
    config::load(Path::new(&cli.config))?;

    let conf = config::get();
    let bind = conf.api.bind.parse()?;

    Ok(())
}
