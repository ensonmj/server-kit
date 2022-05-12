use crate::{tracer, Result};

pub(crate) const BUF_SIZE: usize = 20480;

pub fn setup() -> Result<()> {
    dotenv::dotenv().ok();
    tracer::setup()
}

pub fn teardown() {
    tracer::teardown()
}
