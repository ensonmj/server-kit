use anyhow::Result;
use tracing::{debug, instrument};

use server_kit::protocol::nshead::Nshead;
use server_kit::{global, Server};
use server_kit::{Handler, Message};

#[tokio::main]
async fn main() -> Result<()> {
    global::setup()?;

    let mut server = Server::new("./conf/server.toml").await?;
    let handler = Handler::<Nshead, _>::new(Box::new(echo));
    server.with_service(handler);

    server.start().await?;

    global::teardown();
    Ok(())
}

#[instrument(skip_all)]
async fn echo(buf: Message) -> server_kit::Result<Message> {
    debug!("process message");
    Ok(buf)
}
