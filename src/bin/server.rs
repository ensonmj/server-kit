use anyhow::Result;

use server_kit::Server;
use tracing::debug;

use server_kit::protocol::nshead;
use server_kit::Handler;

#[tokio::main]
async fn main() -> Result<()> {
    let p = nshead::Nshead::default();
    let handler = Handler::new(
        Box::new(p),
        Box::new(|buf| {
            debug!("write message");
            Ok(buf.to_vec())
        }),
    );
    Ok(Server::start_server(handler).await?)
}
