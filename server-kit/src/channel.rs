use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::instrument;

use crate::{protocol::Protocol, Result};

pub struct Channel {
    addr: String,
    protocol: Box<dyn Protocol + Sync + Send + 'static>,
}

impl Channel {
    pub fn new(addr: String, protocol: Box<dyn Protocol + Sync + Send + 'static>) -> Self {
        Self { addr, protocol }
    }

    #[instrument(name = "channel", skip_all)]
    pub async fn process(&mut self, req: &[u8]) -> Result<Vec<u8>> {
        let mut stream = TcpStream::connect(&self.addr).await?;

        // pack request
        let buf = self.protocol.pack_request(req);
        // send request
        stream.write_all(&buf).await?;

        // read response
        let mut buffer = BytesMut::with_capacity(4096);
        stream.read_buf(&mut buffer).await?;

        // parse response
        let buf = self.protocol.parse(&buffer)?;

        // process response
        let buf = self.protocol.process_response(buf)?;
        Ok(buf.to_vec())
    }
}
