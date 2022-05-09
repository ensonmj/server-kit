use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use futures_util::Future;
use opentelemetry::global;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tracing::instrument;
use tracing::Instrument;
use tracing::{debug, error, trace_span, warn};

use crate::conf::{self, Conf};
use crate::handler::Handler;
use crate::socket::Socket;
use crate::Result;

pub struct Server<Fut>
where
    Fut: Future<Output = Result<Vec<u8>>> + Sync + Send + 'static,
{
    conf: Conf,
    handler: Option<Arc<Handler<Fut>>>,
}

impl<Fut> Server<Fut>
where
    Fut: Future<Output = Result<Vec<u8>>> + Sync + Send + 'static,
{
    pub async fn new(conf: impl AsRef<Path>) -> Result<Self> {
        let conf: Conf = conf::read_conf(conf).await?;
        Ok(Self {
            conf,
            handler: None,
        })
    }

    pub fn with_service(&mut self, handler: Handler<Fut>) {
        self.handler = Some(Arc::new(handler));
    }

    #[instrument(skip_all)]
    pub async fn start(&mut self) -> Result<()> {
        let addr = format!("127.0.0.1:{}", self.conf.port);
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

        // sending remaining spans
        global::shutdown_tracer_provider();

        Ok(())
    }

    #[instrument(skip_all)]
    async fn process(&self, addr: SocketAddr, stream: TcpStream) -> Result<()> {
        let handler = Arc::clone(self.handler.as_ref().unwrap());
        if let Err(e) = tokio::spawn(
            async move {
                let mut socket = Socket::new(addr, stream);
                socket.with_handler(handler);
                socket.process().await
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
