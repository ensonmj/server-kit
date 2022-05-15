use anyhow::Result;
use echo_nshead::echo::EchoRequest;
use echo_nshead::EchoService;
use echo_nshead::EchoStub;
use serde_derive::Deserialize;
use tracing::debug;

use server_kit::channel::Channel;
use server_kit::conf;
use server_kit::global;
use server_kit::protocol::Nshead;

#[derive(Deserialize)]
struct Conf {
    pub ip: String,
    pub port: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    global::setup()?;

    let conf: Conf = conf::read_conf("./conf/client.toml").await?;
    let addr = format!("{}:{}", &conf.ip, conf.port);

    let channel = Channel::<Nshead>::new(addr);
    let stub = EchoStub::new(channel);

    let mut req = EchoRequest::new();
    req.set_message("hello".to_string());
    let resp = stub.echo(req).await?;

    debug!("Receive data: {resp:?}");

    global::teardown();
    Ok(())
}
