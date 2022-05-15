use anyhow::Result;
use tracing::{debug, instrument};

use echo_nshead::echo::{EchoRequest, EchoResponse};
use echo_nshead::EchoServiceImpl;
use server_kit::protocol::Nshead;
use server_kit::{global, Server};

#[tokio::main]
async fn main() -> Result<()> {
    global::setup()?;

    let mut server = Server::new("./conf/server.toml").await?;

    let service = EchoServiceImpl::new(echo);
    server.add_service::<Nshead, _>(service)?;

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
