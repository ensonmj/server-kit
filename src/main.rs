use std::time::Duration;

use opentelemetry::global;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

mod logger;
mod tracer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    dotenv::dotenv().ok();
    let tracer = tracer::init()?;
    logger::init(tracer);

    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    loop {
        match listener.accept().await {
            Err(e) => {
                tracing::error!("couldn't get client: {:?}", e);
                break;
            }
            Ok((stream, _addr)) => {
                if let Err(e) = tokio::spawn(process(stream)).await {
                    tracing::warn!("process err:{}", e)
                }
            }
        }
    }

    // sending remaining spans
    global::shutdown_tracer_provider();

    Ok(())
}

#[tracing::instrument(name = "worker")]
async fn process(mut stream: TcpStream) {
    tracing::debug!("process connection");

    let mut buffer = [0; 32];
    let buffer = read(&mut stream, &mut buffer).await;

    write(&mut stream, buffer).await;
}

#[tracing::instrument(skip(buffer))]
async fn read<'buf>(stream: &mut TcpStream, buffer: &'buf mut [u8]) -> &'buf [u8] {
    tracing::debug!("read message");
    tokio::time::sleep(Duration::from_millis(10)).await;
    if let Ok(n) = stream.read(buffer).await {
        &buffer[..n]
    } else {
        buffer
    }
}

#[tracing::instrument]
async fn write(stream: &mut TcpStream, buffer: &[u8]) {
    tracing::debug!("write message");
    tokio::time::sleep(Duration::from_millis(10)).await;
    let _ = stream.write_all(buffer).await;
}
