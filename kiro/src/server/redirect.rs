// src/server/redirect.rs
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

use axum::{
    http::Uri,
    response::{IntoResponse, Redirect},
    Router,
};
use std::{future::Future, net::SocketAddr};

use crate::config;

pub async fn redirect_http_to_https<F>(ports: config::Ports, signal: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    async fn handler(uri: Uri, ports: config::Ports) -> impl IntoResponse {
        let https_uri = format!(
            "https://localhost:{}{}",
            ports.https(),
            uri.path_and_query().map_or("", |x| x.as_str())
        );
        tracing::debug!("Redirecting {} to {}", uri, https_uri);
        Redirect::permanent(&https_uri)
    }

    let app = Router::new().fallback(move |uri: Uri| handler(uri, ports));

    let addr = SocketAddr::from(([0, 0, 0, 0], ports.http()));
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            tracing::info!("🔄 HTTP redirect server listening on {}", addr);
            axum::serve(listener, app.into_make_service())
                .with_graceful_shutdown(signal)
                .await
                .unwrap_or_else(|e| tracing::error!("HTTP redirect server error: {}", e));
        }
        Err(e) => {
            tracing::error!(
                "Failed to start HTTP redirect server on port {}: {}",
                ports.http(),
                e
            );
        }
    }
}
