use tracing::instrument;

use crate::protocol::Protocol;
use crate::Result;

pub struct Handler {
    protocol: Box<dyn Protocol + Sync + Send + 'static>,
    process_fn: Box<dyn Fn(&[u8]) -> Result<Vec<u8>> + Sync + Send + 'static>,
}

impl Handler {
    pub fn new(
        protocol: Box<dyn Protocol + Sync + Send + 'static>,
        process_fn: Box<dyn Fn(&[u8]) -> Result<Vec<u8>> + Sync + Send + 'static>,
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
    pub fn process(&self, buf: &[u8]) -> Result<Vec<u8>> {
        let buf = self.protocol.process_request(buf)?;
        let buf = (self.process_fn)(&buf)?;
        Ok(self.protocol.pack_response(&buf))
    }
}
