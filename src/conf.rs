use std::path::Path;

use serde::de::DeserializeOwned;
use tokio::{fs::File, io::AsyncReadExt};

use crate::Result;

pub async fn read_conf<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: DeserializeOwned,
{
    let mut file = File::open(path).await?;
    let mut content = String::new();
    file.read_to_string(&mut content).await?;
    Ok(toml::from_str(&content)?)
}
