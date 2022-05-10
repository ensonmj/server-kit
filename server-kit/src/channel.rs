use tokio::{io::AsyncWriteExt, net::TcpStream};
use tracing::instrument;

use crate::{protocol::Protocol, Message, Result};

pub struct Channel {
    addr: String,
    protocol: Box<dyn Protocol + Sync + Send + 'static>,
}

impl Channel {
    pub fn new(addr: String, protocol: Box<dyn Protocol + Sync + Send + 'static>) -> Self {
        Self { addr, protocol }
    }

    #[instrument(name = "channel", skip_all)]
    pub async fn process(&mut self, req: Message) -> Result<Vec<u8>> {
        let mut stream = TcpStream::connect(&self.addr).await?;

        // pack request
        let buf = self.protocol.pack_request(req);
        // send request
        stream.write_all(&buf).await?;

        // parse response
        let buf = self.protocol.parse(&mut stream).await?;

        // process response
        let buf = self.protocol.process_response(buf)?;
        Ok(buf.to_vec())
    }
}
