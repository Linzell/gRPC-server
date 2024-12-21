// utils/telemetry.rs
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

use std::str::FromStr;

use kiro_database::get_env_or;
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

use crate::{error::ServerError, version};

pub struct MetadataMap<'a>(pub &'a tonic::metadata::MetadataMap);

impl<'a> Extractor for MetadataMap<'a> {
    /// Get a value for a key from the MetadataMap.  If the value can't be converted to &str, returns None
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|metadata| metadata.to_str().ok())
    }

    /// Collect all the keys from the MetadataMap.
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
pub fn init_tracer() -> Result<(), ServerError> {
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
                        format!("kiro/api-client-{}", get_env_or("ENVIRONMENT", "DEV")),
                    ),
                    opentelemetry::KeyValue::new("service.version", version!("v")),
                ])),
        )
        .install_batch(Tokio)?;

    global::set_tracer_provider(tracer_provider.clone());

    tracing::trace!(target: "relay", "✅ Successfully initialized trace provider on tokio runtime");

    let filter = filter::Targets::new()
        .with_target("kiro", Level::from_str("INFO").unwrap())
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

    tracing::trace!("🔍 Tracer initialized");

    Ok(())
}
