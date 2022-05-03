use opentelemetry::{sdk::trace::Tracer, trace::TraceError};

pub fn init() -> Result<Tracer, TraceError> {
    opentelemetry_jaeger::new_pipeline()
        .with_service_name("server-kit")
        .install_batch(opentelemetry::runtime::Tokio)
}
