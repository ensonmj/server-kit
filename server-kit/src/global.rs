use crate::{tracer, Result};

pub fn init() -> Result<()> {
    dotenv::dotenv().ok();
    tracer::init()
}
