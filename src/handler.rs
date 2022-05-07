use futures_util::Future;
use tracing::instrument;

use crate::protocol::Protocol;
use crate::Result;

pub struct Handler<Fut>
where
    Fut: Future<Output = Result<Vec<u8>>> + Sync + Send + 'static,
{
    protocol: Box<dyn Protocol + Sync + Send + 'static>,
    process_fn: Box<dyn Fn(Vec<u8>) -> Fut + Sync + Send + 'static>,
}

impl<Fut> Handler<Fut>
where
    Fut: Future<Output = Result<Vec<u8>>> + Sync + Send + 'static,
{
    pub fn new(
        protocol: Box<dyn Protocol + Sync + Send + 'static>,
        process_fn: Box<dyn Fn(Vec<u8>) -> Fut + Sync + Send + 'static>,
    ) -> Self {
        Self {
            protocol,
            process_fn,
        }
    }

    #[instrument(skip_all)]
    pub fn parse<'buf>(&self, buf: &'buf [u8]) -> Result<&'buf [u8]> {
        self.protocol.parse(buf)
    }

    #[instrument(skip_all)]
    pub async fn process(&self, buf: &[u8]) -> Result<Vec<u8>> {
        let buf = self.protocol.process_request(buf)?;
        let buf = (self.process_fn)(buf).await?;
        Ok(self.protocol.pack_response(&buf))
    }
}
