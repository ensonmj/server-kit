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

#[derive(Default)]
pub struct Socket<Fut>
where
    Fut: Future<Output = Result<Vec<u8>>> + Sync + Send + 'static,
{
    pub stream: Option<TcpStream>,

    pub read_buf: Vec<u8>,
    pub msg_buf: Vec<u8>,

    handler: Option<Arc<Handler<Fut>>>,
}

impl<Fut> Socket<Fut>
where
    Fut: Future<Output = Result<Vec<u8>>> + Sync + Send + 'static,
{
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: Some(stream),
            read_buf: Default::default(),
            msg_buf: Default::default(),
            handler: None,
            // ..Default::default()
        }
    }

    pub fn with_handler(&mut self, h: Arc<Handler<Fut>>) {
        self.handler = Some(h);
    }

    #[instrument(name = "worker", skip_all)]
    pub async fn process(&mut self) -> Result<()> {
        let mut buffer = BytesMut::with_capacity(4096);
        self.stream.as_mut().unwrap().read_buf(&mut buffer).await?;
        debug!("{buffer:?}");
        let buf = self.handler.as_ref().unwrap().parse(&buffer)?;
        debug!("{buf:?}");
        let msg = self.handler.as_ref().unwrap().process(buf).await?;

        debug!("write message");
        Ok(self.stream.as_mut().unwrap().write_all(&msg).await?)
    }
}
