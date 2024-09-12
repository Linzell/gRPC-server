// utils/telemetry.rs

use std::{str::FromStr, sync::Arc};

use opentelemetry::propagation::Extractor;
use opentelemetry_otlp::WithExportConfig;
use tonic::{metadata::MetadataValue, transport::Channel};
use tracing::Level;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};

use super::env::get_env_or;

use crate::{config::Configuration, version};

pub struct MetadataMap<'a>(pub &'a tonic::metadata::MetadataMap);

impl<'a> Extractor for MetadataMap<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|metadata| metadata.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|key| match key {
                tonic::metadata::KeyRef::Ascii(v) => v.as_str(),
                tonic::metadata::KeyRef::Binary(v) => v.as_str(),
            })
            .collect::<Vec<_>>()
    }
}

/// # Init tracer
///
/// The `init_tracer` method initializes the tracer.
///
/// ```rust
/// init_tracer();
///
/// println!("🔍 Tracer initialized");
/// ```
pub fn init_tracer(conf: &Arc<Configuration>) {
    let jaeger_endpoint = get_env_or("JAEGER_AGENT_HOST", "http://localhost:4317");

    let mut metadata = tonic::metadata::MetadataMap::new();
    metadata.insert("x-custom-header", MetadataValue::from_static("value"));

    let channel = Channel::from_shared(jaeger_endpoint.clone())
        .unwrap()
        .connect_timeout(std::time::Duration::from_secs(5))
        .connect_lazy();

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(jaeger_endpoint)
        .with_metadata(metadata)
        .with_channel(channel);

    let filter = filter::Targets::new()
        .with_target("kiro", Level::from_str(&conf.logging.level).unwrap())
        .with_default(Level::INFO);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(opentelemetry_sdk::trace::config().with_resource(
            opentelemetry_sdk::Resource::new(vec![
                opentelemetry::KeyValue::new(
                    "service.name",
                    format!("kiro-{}", get_env_or("ENVIRONMENT", "PRD")),
                ),
                opentelemetry::KeyValue::new("service.version", version!("v")),
                opentelemetry::KeyValue::new("host.os", std::env::consts::OS),
                opentelemetry::KeyValue::new("host.architecture", std::env::consts::ARCH),
            ]),
        ))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true),
        )
        .init();

    tracing::info!("🔍 Tracer initialized");
}
