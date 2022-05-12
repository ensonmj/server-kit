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
    // _marker: PhantomData<P>,
    protocol: P,
}

impl<P> Channel<P>
where
    P: Protocol,
{
    pub fn new(addr: String, protocol: P) -> Self {
        Self {
            addr,
            // _marker: PhantomData,
            protocol,
        }
    }

    #[instrument(name = "channel", skip_all)]
    pub async fn process(&self, req: CommonMsg) -> Result<Vec<u8>> {
        let mut stream = TcpStream::connect(&self.addr).await?;

        // pack request
        let buf = self.protocol.pack_request(req);
        // send request
        stream.write_all(&buf).await?;

        // parse response
        let buf = self.protocol.parse(&mut stream).await?;

        // process response
        self.protocol.process_response(buf).await
    }
}
