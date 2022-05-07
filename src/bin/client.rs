use anyhow::Result;
use opentelemetry::global;
use serde_derive::Deserialize;
use server_kit::channel::Channel;
use server_kit::conf;
use tracing::debug;
use tracing::instrument;

use server_kit::logger;
use server_kit::protocol::nshead;
use server_kit::tracer;

#[derive(Deserialize)]
struct Conf {
    pub port: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let tracer = tracer::init()?;
    logger::init(tracer);

    let conf: Conf = conf::read_conf("./conf/client.toml").await?;
    let addr = format!("127.0.0.1:{}", conf.port);

    let protocol = Box::new(nshead::Nshead::default());
    let mut channel = Channel::new(addr, protocol);
    let buf = channel.process(echo).await?;
    debug!("Receive data: {buf:?}");

    // sending remaining spans
    global::shutdown_tracer_provider();
    Ok(())
}

#[instrument(skip_all)]
async fn echo() -> server_kit::Result<Vec<u8>> {
    let payload = b"hello";
    debug!("Send data:{payload:?}");
    Ok(payload.to_vec())
}
