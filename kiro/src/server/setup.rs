// src/server/setup.rs
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

use axum::routing::get;
use axum_server::tls_rustls::RustlsConfig;
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderName, HeaderValue, Method,
};
use std::io;
use tonic_health::server::HealthReporter;
use tower_http::cors::{AllowOrigin, CorsLayer};

#[cfg(feature = "governors")]
use std::sync::Arc;
#[cfg(feature = "governors")]
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

#[cfg(feature = "documentation")]
use utoipa::OpenApi;
#[cfg(feature = "documentation")]
use utoipa_swagger_ui::SwaggerUi;

#[cfg(feature = "documentation")]
use crate::utils::doc::ApiDoc;

#[cfg(feature = "client")]
use kiro_client::{
    auth_routes, user_routes, AuthService, AuthServiceServer, ClientService, ClientServiceServer,
};

#[cfg(feature = "client")]
use crate::middleware::auth::auth_layer;
#[cfg(feature = "tracing")]
use crate::middleware::logging::trace_layer;

use super::{health, Database};

pub async fn create_tls_config() -> Result<RustlsConfig, io::Error> {
    let cert = tokio::fs::read("certs/cert.pem").await?;
    let key = tokio::fs::read("certs/key.pem").await?;
    let config = RustlsConfig::from_pem(cert, key).await?;

    #[cfg(feature = "tracing")]
    tracing::info!("🔐 TLS configuration loaded");

    Ok(config)
}

pub async fn setup_health_reporter(health_reporter: &mut HealthReporter) {
    // health_reporter
    //     .set_serving::<AdminServiceServer<AdminService>>()
    //     .await;
    #[cfg(feature = "client")]
    {
        health_reporter
            .set_serving::<AuthServiceServer<AuthService>>()
            .await;
        health_reporter
            .set_serving::<ClientServiceServer<ClientService>>()
            .await;
    }
    // health_reporter
    //     .set_serving::<TicketingServiceServer<TicketingService<Database>>>()
    //     .await;
    // health_reporter
    //     .set_serving::<StripeServiceServer<StripeService<StripeClient>>>()
    //     .await;
    // health_reporter
    //     .set_serving::<StorageServiceServer<StorageService>>()
    //     .await;

    #[cfg(feature = "tracing")]
    tracing::info!("🫀 Health service is running");
}

pub async fn create_app(
    db: Database, config: crate::config::Config,
) -> Result<axum::Router, crate::error::ServerError> {
    let cors = setup_cors(config.clone())?;
    let mut routes_builder = setup_routes(db.clone()).await?.routes().into_axum_router();
    let openapi = ApiDoc::openapi();

    // Add governors
    #[cfg(feature = "governors")]
    {
        let governor_conf = Arc::new(
            GovernorConfigBuilder::default()
                .per_second(1)
                .burst_size(100)
                .finish()
                .unwrap(),
        );

        routes_builder = routes_builder.layer(GovernorLayer {
            config: governor_conf,
        });
    }

    #[cfg(feature = "client")]
    {
        routes_builder = routes_builder.layer(auth_layer(db.clone()));
    }

    #[cfg(feature = "tracing")]
    {
        routes_builder = routes_builder.layer(trace_layer(&config));
    }

    #[cfg(feature = "documentation")]
    {
        routes_builder = routes_builder
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi));
    }

    routes_builder = routes_builder
        .layer(cors)
        .route("/health", get(health::health_check))
        .route("/", get(|| async { "Hello, World!" }));

    #[cfg(feature = "client")]
    {
        routes_builder = routes_builder
            .nest("/auth", auth_routes(db.clone()))
            .nest("/user", user_routes(db));
    }

    Ok(routes_builder)
}

fn setup_cors(config: crate::config::Config) -> Result<CorsLayer, crate::error::ServerError> {
    let frontend_url = config.app.frontend_url.clone();

    Ok(CorsLayer::new()
        .allow_headers([
            AUTHORIZATION,
            ACCEPT,
            CONTENT_TYPE,
            HeaderName::from_static("grpc-status"),
            HeaderName::from_static("grpc-message"),
            HeaderName::from_static("grpc-encoding"),
            HeaderName::from_static("grpc-accept-encoding"),
            HeaderName::from_static("x-grpc-web"),
            HeaderName::from_static("x-user-agent"),
        ])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_origin(AllowOrigin::predicate(move |origin: &HeaderValue, _| {
            origin
                .to_str()
                .map(|origin_string| {
                    origin_string == frontend_url
                        || origin_string == "https://localhost:5173"
                        || origin_string == "http://localhost:5173"
                        || origin_string == "https://localhost"
                        || origin_string == "http://localhost"
                })
                .unwrap_or(false)
        }))
        .allow_credentials(true)
        .allow_private_network(true)
        .expose_headers(vec![
            HeaderName::from_static("grpc-status"),
            HeaderName::from_static("grpc-message"),
            HeaderName::from_static("grpc-encoding"),
            HeaderName::from_static("grpc-accept-encoding"),
        ]))
}

async fn setup_routes(
    db: Database,
) -> Result<tonic::service::RoutesBuilder, crate::error::ServerError> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    setup_health_reporter(&mut health_reporter).await;

    let reflection_service = crate::server::reflection::setup_reflection_service();

    #[cfg(feature = "tracing")]
    tracing::info!("🪞 Reflection service is running");

    let mut routes_builder = tonic::service::Routes::builder();

    routes_builder
        .add_service(reflection_service)
        .add_service(tonic_web::enable(health_service));

    #[cfg(feature = "client")]
    routes_builder
        .add_service(tonic_web::enable(AuthService::build(db.clone())))
        .add_service(tonic_web::enable(ClientService::build(db.clone())));

    Ok(routes_builder)
}
