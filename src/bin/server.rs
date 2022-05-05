use std::time::Duration;

use opentelemetry::global;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use server_kit::logger;
use server_kit::nshead;
use server_kit::tracer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    dotenv::dotenv().ok();
    let tracer = tracer::init()?;
    logger::init(tracer);

    let listener = TcpListener::bind("127.0.0.1:8787").await?;
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
async fn process(
    mut stream: TcpStream,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    tracing::debug!("process connection");

    let buffer = read(&mut stream).await?;

    write(&mut stream, &buffer).await
}

#[tracing::instrument(skip_all)]
async fn read<'buf>(
    stream: &mut TcpStream,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    tracing::debug!("read message");
    tokio::time::sleep(Duration::from_millis(10)).await;

    let mut data = [0; nshead::NSHEAD_LEN];
    stream.read_exact(&mut data).await?;
    let head = nshead::Nshead::from_u8_slice(&data);
    if head.magic_num != nshead::NSHEAD_MAGICNUM {
        tracing::warn!("Unexpected header: {:?}", head);
        return Err(format!("unexpected header magic_num[{}]", head.magic_num).into());
    }
    tracing::debug!("Receive header: {:?}", head);

    let mut payload = vec![0; head.body_len as usize];
    stream.read_exact(&mut payload).await?;
    tracing::debug!("Receive data:[{:?}]", payload);

    Ok(payload)
}

#[tracing::instrument(skip_all)]
async fn write(
    stream: &mut TcpStream,
    buffer: &[u8],
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    tracing::debug!("write message");
    tokio::time::sleep(Duration::from_millis(10)).await;

    let mut head = nshead::Nshead::default();
    head.body_len = buffer.len() as u32;
    tracing::debug!("header[{:?}]", &head);
    let head = head.as_u8_slice();
    stream.write_all(head).await?;

    Ok(stream.write_all(buffer).await?)
}
