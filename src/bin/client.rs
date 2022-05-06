use anyhow::Result;
use opentelemetry::global;
use serde_derive::Deserialize;
use server_kit::conf;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use server_kit::logger;
use server_kit::nshead;
use server_kit::tracer;

#[derive(Deserialize)]
struct Conf {
    pub port: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let tracer = tracer::init()?;
    logger::init(tracer);

    let conf: Conf = conf::read_conf("./conf/conf.toml").await?;
    let addr = format!("127.0.0.1:{}", conf.port);

    process(&addr).await?;

    // sending remaining spans
    global::shutdown_tracer_provider();
    Ok(())
}

#[tracing::instrument(name = "client")]
async fn process(addr: &str) -> Result<()> {
    let payload = b"hello";
    let mut head = nshead::Nshead::default();
    head.body_len = payload.len() as u32;
    tracing::debug!("header[{:?}]", &head);
    let head = head.as_u8_slice();

    let mut stream = TcpStream::connect(addr).await?;
    tracing::debug!("Successfully connected to server");

    stream.write_all(head).await?;
    tracing::debug!("Sent header...");
    stream.write_all(payload).await?;
    tracing::debug!("Sent payload...");

    let mut data = [0; nshead::NSHEAD_LEN];
    stream.read_exact(&mut data).await?;
    let head = nshead::Nshead::from_u8_slice(&data);
    if head.magic_num != nshead::NSHEAD_MAGICNUM {
        tracing::warn!("Unexpected header: {:?}", head);
        return Err(server_kit::Error::MagicNum(format!(
            "unexpected header magic_num[{}]",
            head.magic_num
        ))
        .into());
    }
    tracing::debug!("Receive header: {:?}", head);

    let mut payload = vec![0; head.body_len as usize];
    stream.read_exact(&mut payload).await?;
    tracing::debug!("Receive data:[{:?}]", payload);

    Ok(())
}
