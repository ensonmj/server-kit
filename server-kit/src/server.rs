use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tracing::instrument;
use tracing::Instrument;
use tracing::{debug, error, trace_span, warn};

use crate::conf::{self, Conf};
use crate::service::ServiceManger;
use crate::socket::Socket;
use crate::Result;
use crate::Service;

pub struct Server {
    conf: Conf,
    svc_manager: Arc<ServiceManger>,
}

impl Server {
    pub async fn new(conf: impl AsRef<Path>) -> Result<Self> {
        let conf: Conf = conf::read_conf(conf).await?;
        Ok(Self {
            conf,
            svc_manager: Default::default(),
        })
    }

    pub fn add_service<S>(&mut self, svc: S) -> Result<()>
    where
        S: Service,
    {
        Arc::get_mut(&mut self.svc_manager)
            .map(|m| m.add_service(svc))
            .unwrap()
    }

    #[instrument(skip_all)]
    pub async fn start(&mut self) -> Result<()> {
        let addr = format!("{}:{}", &self.conf.ip, self.conf.port);
        debug!("start server on {addr}");
        let listener = TcpListener::bind(&addr).await?;
        loop {
            match listener.accept().instrument(trace_span!("accept")).await {
                Err(e) => {
                    error!("couldn't get client: {:?}", e);
                    break;
                }
                Ok((stream, addr)) => {
                    if let Err(e) = self.process(addr, stream).await {
                        warn!("process err:{}", e)
                    }
                }
            }
        }

        Ok(())
    }

    #[instrument(level = "trace", skip_all)]
    async fn process(&self, addr: SocketAddr, stream: TcpStream) -> Result<()> {
        let svc_manager = Arc::clone(&self.svc_manager);
        if let Err(e) = tokio::spawn(
            async move {
                let mut socket = Socket::new(addr, stream);
                socket.process(svc_manager).await
            }
            .instrument(trace_span!("worker")),
        )
        .await
        {
            warn!("process err:{}", e)
        }

        Ok(())
    }
}
