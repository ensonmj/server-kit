use anyhow::Result;
use tracing::{debug, instrument};

use server_kit::protocol::nshead;
use server_kit::Handler;
use server_kit::{global, Server};

#[tokio::main]
async fn main() -> Result<()> {
    global::init()?;

    let p = nshead::Nshead::default();
    let handler = Handler::new(
        Box::new(p),
        Box::new(|buf| async move {
            debug!("process message");
            Ok(buf)
        }),
    );

    let mut server = Server::new("./conf/server.toml").await?;
    server.with_service(handler);

    Ok(server.start().await?)
}

#[instrument(skip_all)]
async fn echo(buf: &[u8]) -> server_kit::Result<Vec<u8>> {
    debug!("process message");
    Ok(buf.to_vec())
}
