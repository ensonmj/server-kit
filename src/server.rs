use std::sync::Arc;

use opentelemetry::global;
use serde_derive::Deserialize;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tracing::instrument;
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

pub struct Server {
    conf: Conf,
    handler: Arc<Handler>,
}

impl Server {
    pub async fn start_server(handler: Handler) -> Result<()> {
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

    #[instrument(name = "worker", skip_all)]
    async fn process(&self, stream: TcpStream) -> Result<()> {
        debug!("process connection");

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
