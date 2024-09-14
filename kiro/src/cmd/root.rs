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
        "Starting up... 🚀"
    );

    let mut signals = Signals::new([SIGINT, SIGTERM])?;

    if let Some(signal) = signals.next().await {
        warn!(signal = ?signal, "Signal received, terminating process 🛑");
    }

    tracing::info!("Shutting down... 🛑");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::runtime::Runtime;
    use tokio::time::timeout;

    #[test]
    fn test_run() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let result = timeout(Duration::from_secs(1), run()).await;
            assert!(result.is_err(), "Test did not complete within 1 seconds");
        });
    }
}
