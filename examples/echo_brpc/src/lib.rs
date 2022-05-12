use async_trait::async_trait;

use echo::{EchoRequest, EchoResponse};
use server_kit::{Result, Service, ServiceDescriptor};

include!(env!("SCHEMA_MOD_RS"));

mod echo_impl;
mod echo_stub;

pub use echo_impl::EchoServiceImpl;
pub use echo_stub::EchoStub;

// EchoService
#[async_trait]
pub trait EchoService: Service {
    async fn echo(&self, buf: EchoRequest) -> Result<EchoResponse>;
    async fn another_echo(&self, buf: EchoRequest) -> Result<EchoResponse>;
}

pub struct EchoSeviceDescriptor {}

impl ServiceDescriptor for EchoSeviceDescriptor {
    fn name(&self) -> &'static str {
        "echo_brpc"
    }

    fn full_name(&self) -> &'static str {
        "example.echo_brpc"
    }
}
