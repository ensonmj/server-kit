use bytes::BytesMut;
use futures_util::Future;
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
    pub async fn process<F, Fut>(&mut self, f: F) -> Result<Vec<u8>>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<Vec<u8>>>,
    {
        let mut stream = TcpStream::connect(&self.addr).await?;

        // create message
        let msg = f().await?;

        // send request
        let buf = self.protocol.pack_request(&msg);
        stream.write_all(&buf).await?;

        // read response
        let mut buffer = BytesMut::with_capacity(4096);
        stream.read_buf(&mut buffer).await?;
        let buf = self.protocol.parse(&buffer)?;
        Ok(buf.to_vec())
    }
}
