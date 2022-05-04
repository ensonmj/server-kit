use opentelemetry::sdk::trace::Tracer;
use time::macros::format_description;
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

pub fn init(tracer: Tracer) {
    let timer = LocalTime::new(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
    ));
    // Initialize `tracing` using `opentelemetry-tracing` and configure logging
    Registry::default()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_timer(timer))
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();
}
