use std::net::SocketAddr;
use std::sync::Arc;

use bytes::BytesMut;
use futures_util::Future;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::debug;
use tracing::instrument;

use crate::handler::Handler;
use crate::Result;

pub struct Socket<Fut>
where
    Fut: Future<Output = Result<Vec<u8>>> + Sync + Send + 'static,
{
    pub addr: SocketAddr,
    pub stream: TcpStream,

    pub read_buf: Vec<u8>,
    pub msg_buf: Vec<u8>,

    handler: Option<Arc<Handler<Fut>>>,
}

impl<Fut> Socket<Fut>
where
    Fut: Future<Output = Result<Vec<u8>>> + Sync + Send + 'static,
{
    pub fn new(addr: SocketAddr, stream: TcpStream) -> Self {
        Self {
            addr,
            stream,
            read_buf: Default::default(),
            msg_buf: Default::default(),
            handler: None,
        }
    }

    pub fn with_handler(&mut self, h: Arc<Handler<Fut>>) {
        self.handler = Some(h);
    }

    #[instrument(name = "worker", skip_all, fields(remote_addr = %self.addr))]
    pub async fn process(&mut self) -> Result<()> {
        // read request
        let mut buffer = BytesMut::with_capacity(4096);
        self.stream.read_buf(&mut buffer).await?;
        debug!("{buffer:?}");

        // parse request
        let buf = self.handler.as_ref().unwrap().parse(&buffer)?;
        debug!("{buf:?}");

        // process request
        let msg = self.handler.as_ref().unwrap().process(buf).await?;

        // write response
        debug!("write message");
        Ok(self.stream.write_all(&msg).await?)
    }
}
