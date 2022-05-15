use anyhow::Result;
use echo_brpc::echo::EchoRequest;
use echo_brpc::EchoService;
use serde_derive::Deserialize;
use tracing::debug;

use server_kit::channel::Channel;
use server_kit::conf;
use server_kit::global;
use server_kit::protocol::Brpc;

use echo_brpc::EchoStub;

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

    let channel = Channel::<Brpc>::new(addr);
    let stub = EchoStub::new(channel);

    let mut req = EchoRequest::new();
    req.set_message("hello".to_string());
    let resp = stub.echo(req).await?;
    debug!("Receive data: {resp:?}");

    let mut req = EchoRequest::new();
    req.set_message("another hello".to_string());
    let resp = stub.another_echo(req).await?;
    debug!("Receive data: {resp:?}");

    global::teardown();
    Ok(())
}
