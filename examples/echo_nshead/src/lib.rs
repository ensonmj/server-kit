use async_trait::async_trait;

use echo::{EchoRequest, EchoResponse};
use server_kit::{Result, Service};

include!(env!("SCHEMA_MOD_RS"));

mod echo_impl;
mod echo_stub;

pub use echo_impl::EchoServiceImpl;
pub use echo_stub::EchoStub;

// EchoService
#[async_trait]
pub trait EchoService: Service {
    async fn echo(&self, buf: EchoRequest) -> Result<EchoResponse>;
}
