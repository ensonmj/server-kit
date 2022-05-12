use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpStream;
use tracing::instrument;

use crate::service::ServiceManger;
use crate::Result;

pub struct Socket {
    pub addr: SocketAddr,
    pub stream: TcpStream,
}

impl Socket {
    pub fn new(addr: SocketAddr, stream: TcpStream) -> Self {
        Self { addr, stream }
    }

    #[instrument(name = "worker", skip_all, fields(remote_addr = %self.addr))]
    pub async fn process(&mut self, svc_manager: Arc<ServiceManger>) -> Result<()> {
        svc_manager.as_ref().process(&mut self.stream).await
    }
}
