use crate::{logger, tracer, Result};

pub fn init() -> Result<()> {
    dotenv::dotenv().ok();
    let tracer = tracer::init()?;
    logger::init(tracer);
    Ok(())
}
