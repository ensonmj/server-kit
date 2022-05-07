use opentelemetry::sdk::trace::Tracer;

use crate::Result;

pub fn init() -> Result<Tracer> {
    Ok(opentelemetry_jaeger::new_pipeline()
        .with_service_name("server-kit")
        .install_batch(opentelemetry::runtime::Tokio)?)
}
