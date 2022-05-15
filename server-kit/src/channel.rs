use std::marker::PhantomData;

use tokio::{io::AsyncWriteExt, net::TcpStream};
use tracing::instrument;

use crate::message::CommonMsg;
use crate::protocol::Protocol;
use crate::Result;

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
    pub async fn process(&self, req: CommonMsg) -> Result<Vec<u8>> {
        let mut stream = TcpStream::connect(&self.addr).await?;
        let protocol = P::default();

        // pack request
        let buf = protocol.pack_request(req);
        // send request
        stream.write_all(&buf).await?;

        // parse response
        let buf = protocol.parse(&mut stream).await?;

        // process response
        protocol.process_response(buf).await
    }
}
