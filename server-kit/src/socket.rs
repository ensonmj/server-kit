use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::Future;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::debug;
use tracing::instrument;

use crate::handler::Handler;
use crate::protocol::Protocol;
use crate::Message;
use crate::Result;

pub struct Socket<P, Fut>
where
    P: Protocol + Sync + Send + 'static,
    Fut: Future<Output = Result<Message>> + Sync + Send + 'static,
{
    pub addr: SocketAddr,
    pub stream: TcpStream,

    pub read_buf: Vec<u8>,
    pub msg_buf: Vec<u8>,

    handler: Option<Arc<Handler<P, Fut>>>,
}

impl<P, Fut> Socket<P, Fut>
where
    P: Protocol + Sync + Send + 'static,
    Fut: Future<Output = Result<Message>> + Sync + Send + 'static,
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

    pub fn with_handler(&mut self, h: Arc<Handler<P, Fut>>) {
        self.handler = Some(h);
    }

    #[instrument(name = "worker", skip_all, fields(remote_addr = %self.addr))]
    pub async fn process(&mut self) -> Result<()> {
        // parse request
        let buf = self
            .handler
            .as_ref()
            .unwrap()
            .parse(&mut self.stream)
            .await?;
        debug!("read message: {buf:?}");

        // process request
        let msg = self.handler.as_ref().unwrap().process(buf).await?;

        // write response
        debug!("write message: {msg:?}");
        Ok(self.stream.write_all(&msg).await?)
    }
}
