use std::marker::PhantomData;

use futures_util::Future;
use tokio::net::TcpStream;
use tracing::instrument;

use crate::protocol::Protocol;
use crate::{Message, Result};

pub struct Handler<P, Fut>
where
    P: Protocol + Sync + Send + 'static,
    Fut: Future<Output = Result<Message>> + Sync + Send + 'static,
{
    process_fn: Box<dyn Fn(Message) -> Fut + Sync + Send + 'static>,
    _marker: PhantomData<P>,
}

impl<P, Fut> Handler<P, Fut>
where
    P: Protocol + Sync + Send + 'static,
    Fut: Future<Output = Result<Message>> + Sync + Send + 'static,
{
    pub fn new(process_fn: Box<dyn Fn(Message) -> Fut + Sync + Send + 'static>) -> Self {
        Self {
            process_fn,
            _marker: PhantomData,
        }
    }

    #[instrument(skip_all)]
    pub async fn parse(&self, stream: &mut TcpStream) -> Result<Vec<u8>> {
        P::parse(stream).await
    }

    #[instrument(skip_all)]
    pub async fn process(&self, buf: Vec<u8>) -> Result<Vec<u8>> {
        let msg = P::process_request(buf)?;
        let msg = (self.process_fn)(msg).await?;
        Ok(P::pack_response(msg))
    }
}
