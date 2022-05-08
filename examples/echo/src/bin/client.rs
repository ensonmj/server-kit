use anyhow::Result;
use opentelemetry::global;
use serde_derive::Deserialize;
use tracing::debug;
use tracing::instrument;

use server_kit::channel::Channel;
use server_kit::conf;
use server_kit::protocol::nshead;

#[derive(Deserialize)]
struct Conf {
    pub port: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    server_kit::global::init()?;

    let conf: Conf = conf::read_conf("./conf/client.toml").await?;
    let addr = format!("127.0.0.1:{}", conf.port);

    let protocol = Box::new(nshead::Nshead::default());
    let channel = Channel::new(addr, protocol);
    let mut stub = EchoStub::new(channel);

    let req = b"hello";
    let resp = stub.echo(req).await?;
    debug!("Receive data: {resp:?}");

    // sending remaining spans
    global::shutdown_tracer_provider();
    Ok(())
}

struct EchoStub {
    channel: Channel,
}

impl EchoStub {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }

    #[instrument(skip_all)]
    pub async fn echo(&mut self, req: &[u8]) -> server_kit::Result<Vec<u8>> {
        self.channel.process(req).await
    }
}
