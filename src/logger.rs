use opentelemetry::sdk::trace::Tracer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

pub fn init(tracer: Tracer) {
    // Initialize `tracing` using `opentelemetry-tracing` and configure logging
    Registry::default()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();
}
