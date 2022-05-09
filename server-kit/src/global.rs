use crate::{tracer, Result};

pub fn setup() -> Result<()> {
    dotenv::dotenv().ok();
    tracer::setup()
}

pub fn teardown() {
    tracer::teardown()
}
