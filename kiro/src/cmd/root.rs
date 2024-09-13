// cmd/run.rs

use futures::stream::StreamExt;
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook_tokio::Signals;
use tracing::{info, warn};

use crate::utils::error::Error;

pub async fn run() -> Result<(), Error> {
    info!(
        version = env!("CARGO_PKG_VERSION"),
        docs = env!("CARGO_PKG_HOMEPAGE"),
        authors = env!("CARGO_PKG_AUTHORS"),
        "Starting up..."
    );

    let mut signals = Signals::new([SIGINT, SIGTERM])?;
    if let Some(signal) = signals.next().await {
        warn!(signal = ?signal, "Signal received, terminating process");
    }

    Ok(())
}
