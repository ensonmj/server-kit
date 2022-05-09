use std::env;

use opentelemetry::global;
use time::macros::format_description;
use tracing_subscriber::fmt::{self, time::LocalTime};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;
use tracing_tree::HierarchicalLayer;

use crate::Result;

// Initialize `tracing` using `opentelemetry-tracing` and configure logging
pub fn setup() -> Result<()> {
    // tracer
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("server-kit")
        .install_batch(opentelemetry::runtime::Tokio)?;

    // logger
    let env_layer = EnvFilter::from_default_env();

    let timer = LocalTime::new(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
    ));
    let fmt_layer = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_timer(timer);

    let trace_tree_indent = env::var("TRACE_TREE_INDENT")
        .map(|v| v.parse::<usize>())
        .unwrap_or(Ok(2))
        .unwrap_or(2);
    let tree_layer = HierarchicalLayer::new(trace_tree_indent)
        .with_targets(true)
        .with_bracketed_fields(true);

    let tracer_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    Registry::default()
        .with(env_layer)
        .with(fmt_layer)
        .with(tree_layer)
        .with(tracer_layer)
        .init();

    Ok(())
}

pub fn teardown() {
    // sending remaining spans
    global::shutdown_tracer_provider();
}
