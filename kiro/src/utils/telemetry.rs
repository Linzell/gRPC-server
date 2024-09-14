// utils/telemetry.rs

use std::{str::FromStr, sync::Arc};

use opentelemetry::{global, propagation::Extractor, trace::TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime::Tokio,
    trace::{Config, RandomIdGenerator, Sampler},
};
use tonic::{metadata::MetadataValue, transport::Channel};
use tracing::Level;
use tracing_core::Dispatch;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};

use super::{env::get_env_or, error::Error};

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
pub fn init_tracer(conf: &Arc<Configuration>) -> Result<(), Error> {
    let jaeger_endpoint = get_env_or("JAEGER_AGENT_HOST", "http://localhost:4317");

    let mut metadata = tonic::metadata::MetadataMap::new();
    metadata.insert("x-custom-header", MetadataValue::from_static("value"));

    let channel = Channel::from_shared(jaeger_endpoint.clone())
        .unwrap()
        .connect_timeout(std::time::Duration::from_secs(5))
        .connect_lazy();

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(jaeger_endpoint.clone())
        .with_metadata(metadata.clone())
        .with_channel(channel.clone());

    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            Config::default()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(16)
                .with_max_events_per_span(16)
                .with_resource(opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new(
                        "service.name",
                        format!("kiro-{}", get_env_or("ENVIRONMENT", "PRD")),
                    ),
                    opentelemetry::KeyValue::new("service.version", version!("v")),
                ])),
        )
        .install_batch(Tokio)?;

    global::set_tracer_provider(tracer_provider.clone());

    tracing::trace!(target: "relay", "✅ Successfully initialized trace provider on tokio runtime");

    let filter = filter::Targets::new()
        .with_target("kiro", Level::from_str(&conf.logging.level).unwrap())
        .with_default(Level::INFO);

    let dispatch: Dispatch = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("relay")))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true),
        )
        .into();

    dispatch.try_init()?;

    tracing::info!("🔍 Tracer initialized");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_init_tracer() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let conf = Arc::new(Configuration::default());
            init_tracer(&conf).unwrap();
        });

        opentelemetry::global::shutdown_tracer_provider();
    }
}
