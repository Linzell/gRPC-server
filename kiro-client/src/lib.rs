// lib.rs

use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use axum::{response::IntoResponse, routing::get, Router};
use dotenv::dotenv;
use tokio::try_join;
use tracing::log::info;
use tracing::{error_span, Span};

use tonic::transport::{
    /* Certificate,  */ Body, Identity, Server as TonicServer, ServerTlsConfig,
};

use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderName, HeaderValue, Method,
};

use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let conf = config::get();
    let bind = conf.api.bind.parse()?;

    info!(bind = %bind, "Setting up API interface");

    let web = Router::new()
        .fallback(service_static_handler)
        .into_service();

    let grpc = TonicServer::builder()
        .layer(
            CorsLayer::new()
                .allow_headers([
                    AUTHORIZATION,
                    ACCEPT,
                    CONTENT_TYPE,
                    HeaderName::from_static("grpc-status"),
                    HeaderName::from_static("grpc-message"),
                    HeaderName::from_static("x-grpc-web"),
                ])
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_private_network(true),
        )
        .accept_http1(true);

    let backend_handle = tokio::spawn(backend::setup());
    let monitoring_handle = tokio::spawn(monitoring::setup());
    let grpc_handle = tokio::spawn(grpc.serve(bind));

    tokio::spawn(async move {
        if let Err(e) = try_join!(grpc_handle, backend_handle, monitoring_handle) {
            std::process::exit(-1);
        }
    });

    Ok(())
}
