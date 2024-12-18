// src/server/shutdown.rs
//
// Copyright Charlie Cohen <linzellart@gmail.com>
//
// Licensed under the GNU General Public License, Version 3.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.gnu.org/licenses/gpl-3.0.html
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use axum_server::Handle;
use std::time::Duration;
use tokio::signal;

#[cfg(feature = "mailer")]
use crate::{
    config::{ErrorContext, ErrorSeverity},
    utils::error_mailer::ErrorMailer,
};

pub async fn shutdown_signal(
    handle: Handle, #[cfg(feature = "mailer")] config: crate::config::Config,
    #[cfg(not(feature = "mailer"))] _config: crate::config::Config,
) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        "Ctrl+C".to_string()
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
        "SIGTERM".to_string()
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<String>();

    // Wait for shutdown signal
    let signal_type = tokio::select! {
        signal = ctrl_c => signal,
        signal = terminate => signal,
    };

    tracing::info!(
        "🛑 Received {} signal, initiating graceful shutdown",
        signal_type
    );

    #[cfg(feature = "mailer")]
    {
        let error_mailer = ErrorMailer::new(config);

        // Prepare and send shutdown notification
        let context = ErrorContext::builder(ErrorSeverity::High)
            .error_code("SERVER_SHUTDOWN")
            .message(format!(
                "Server shutdown initiated by {} signal",
                signal_type
            ))
            .service("main_server")
            .endpoint("shutdown")
            .error_source("system")
            .request_details(format!(
                "Shutdown initiated at: {}",
                chrono::Utc::now().to_rfc3339()
            ))
            .build();

        // Send shutdown notification email
        if let Err(e) = error_mailer.send_error_notification(&context).await {
            tracing::error!("Failed to send shutdown notification email: {}", e);
        }
    }

    // Initiate graceful shutdown
    handle.graceful_shutdown(Some(Duration::from_secs(10)));

    // Log final shutdown message
    tracing::info!("👋 Server started graceful shutdown");
}
