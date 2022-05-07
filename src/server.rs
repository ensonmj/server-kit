use std::sync::Arc;

use futures_util::Future;
use opentelemetry::global;
use serde_derive::Deserialize;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tracing::{debug, error, warn};

use crate::conf;
use crate::handler::Handler;
use crate::logger;
use crate::socket::Socket;
use crate::tracer;
use crate::Result;

#[derive(Deserialize)]
struct Conf {
    pub port: u32,
}

pub struct Server<Fut>
where
    Fut: Future<Output = Result<Vec<u8>>> + Sync + Send + 'static,
{
    conf: Conf,
    handler: Arc<Handler<Fut>>,
}

impl<Fut> Server<Fut>
where
    Fut: Future<Output = Result<Vec<u8>>> + Sync + Send + 'static,
{
    pub async fn start_server(handler: Handler<Fut>) -> Result<()> {
        dotenv::dotenv().ok();
        let tracer = tracer::init()?;
        logger::init(tracer);

        let conf: Conf = conf::read_conf("./conf/server.toml").await?;
        let server = Server {
            conf,
            handler: Arc::new(handler),
        };

        let addr = format!("127.0.0.1:{}", server.conf.port);
        debug!("start server on {addr}");
        let listener = TcpListener::bind(&addr).await?;
        loop {
            match listener.accept().await {
                Err(e) => {
                    error!("couldn't get client: {:?}", e);
                    break;
                }
                Ok((stream, _addr)) => {
                    if let Err(e) = server.process(stream).await {
                        warn!("process err:{}", e)
                    }
                }
            }
        }

        // sending remaining spans
        global::shutdown_tracer_provider();

        Ok(())
    }

    async fn process(&self, stream: TcpStream) -> Result<()> {
        let handler = Arc::clone(&self.handler);
        if let Err(e) = tokio::spawn(async move {
            let mut socket = Socket::new(stream);
            socket.with_handler(handler);
            socket.process().await
        })
        .await
        {
            warn!("process err:{}", e)
        }

        Ok(())
    }
}
