use anyhow::Result;

use server_kit::Server;
use tracing::{debug, instrument};

use server_kit::protocol::nshead;
use server_kit::Handler;

#[tokio::main]
async fn main() -> Result<()> {
    let p = nshead::Nshead::default();
    let handler = Handler::new(
        Box::new(p),
        Box::new(|buf| async move {
            debug!("process message");
            Ok(buf)
        }),
    );
    Ok(Server::start_server(handler).await?)
}

#[instrument(skip_all)]
async fn echo(buf: &[u8]) -> server_kit::Result<Vec<u8>> {
    debug!("process message");
    Ok(buf.to_vec())
}
