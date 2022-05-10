use futures_util::Future;
use tokio::net::TcpStream;
use tracing::instrument;

use crate::protocol::Protocol;
use crate::{Message, Result};

pub struct Handler<Fut>
where
    Fut: Future<Output = Result<Message>> + Sync + Send + 'static,
{
    protocol: Box<dyn Protocol + Sync + Send + 'static>,
    process_fn: Box<dyn Fn(Message) -> Fut + Sync + Send + 'static>,
}

impl<Fut> Handler<Fut>
where
    Fut: Future<Output = Result<Message>> + Sync + Send + 'static,
{
    pub fn new(
        protocol: Box<dyn Protocol + Sync + Send + 'static>,
        process_fn: Box<dyn Fn(Message) -> Fut + Sync + Send + 'static>,
    ) -> Self {
        Self {
            protocol,
            process_fn,
        }
    }

    #[instrument(skip_all)]
    pub async fn parse(&self, stream: &mut TcpStream) -> Result<Vec<u8>> {
        self.protocol.parse(stream).await
    }

    #[instrument(skip_all)]
    pub async fn process(&self, buf: Vec<u8>) -> Result<Vec<u8>> {
        let msg = self.protocol.process_request(buf)?;
        let msg = (self.process_fn)(msg).await?;
        Ok(self.protocol.pack_response(msg))
    }
}
