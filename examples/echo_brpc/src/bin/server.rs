use anyhow::Result;
use echo_brpc::echo::{EchoRequest, EchoResponse};
use echo_brpc::EchoServiceImpl;
use server_kit::protocol::Brpc;
use tracing::{debug, instrument};


use server_kit::{global, Server};

#[tokio::main]
async fn main() -> Result<()> {
    global::setup()?;

    let mut server = Server::new("./conf/server.toml").await?;
    let service = EchoServiceImpl::new(echo, another_echo);
    server.add_service(Brpc, Box::new(service))?;

    server.start().await?;

    global::teardown();
    Ok(())
}

#[instrument(skip_all)]
async fn echo(req: EchoRequest) -> server_kit::Result<EchoResponse> {
    debug!("echo message");
    let mut resp = EchoResponse::new();
    resp.set_message(req.message().to_string());
    Ok(resp)
}

#[instrument(skip_all)]
async fn another_echo(req: EchoRequest) -> server_kit::Result<EchoResponse> {
    debug!("another echo message");
    let mut resp = EchoResponse::new();
    resp.set_message(req.message().to_string());
    Ok(resp)
}
