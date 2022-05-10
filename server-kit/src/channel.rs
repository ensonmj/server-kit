use std::marker::PhantomData;

use tokio::{io::AsyncWriteExt, net::TcpStream};
use tracing::instrument;

use crate::{protocol::Protocol, Message, Result};

pub struct Channel<P>
where
    P: Protocol,
{
    addr: String,
    _marker: PhantomData<P>,
}

impl<P> Channel<P>
where
    P: Protocol,
{
    pub fn new(addr: String) -> Self {
        Self {
            addr,
            _marker: PhantomData,
        }
    }

    #[instrument(name = "channel", skip_all)]
    pub async fn process(&mut self, req: Message) -> Result<Vec<u8>> {
        let mut stream = TcpStream::connect(&self.addr).await?;

        // pack request
        let buf = P::pack_request(req);
        // send request
        stream.write_all(&buf).await?;

        // parse response
        let buf = P::parse(&mut stream).await?;

        // process response
        let buf = P::process_response(buf)?;
        Ok(buf.to_vec())
    }
}
