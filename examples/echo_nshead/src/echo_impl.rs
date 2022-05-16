use std::future::Future;

use async_trait::async_trait;
use protobuf::Message;
use tracing::instrument;

use server_kit::{
    protocol::{Nshead, Protocol},
    Result, Service, ServiceDescriptor,
};

use crate::{
    echo::{EchoRequest, EchoResponse},
    EchoService,
};

pub struct EchoServiceImpl<F1, Fut1>
where
    F1: Fn(EchoRequest) -> Fut1 + Sync + Send + 'static,
    Fut1: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
{
    echo_fn: F1,
}

impl<F1, Fut1> EchoServiceImpl<F1, Fut1>
where
    F1: Fn(EchoRequest) -> Fut1 + Sync + Send + 'static,
    Fut1: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
{
    pub fn new(echo_fn: F1) -> Self {
        Self { echo_fn }
    }
}

#[async_trait]
impl<F1, Fut1> EchoService for EchoServiceImpl<F1, Fut1>
where
    F1: Fn(EchoRequest) -> Fut1 + Sync + Send + 'static,
    Fut1: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
{
    #[instrument(skip_all)]
    async fn echo(&self, buf: EchoRequest) -> server_kit::Result<EchoResponse> {
        (self.echo_fn)(buf).await
    }
}

#[async_trait]
impl<F1, Fut1> Service for EchoServiceImpl<F1, Fut1>
where
    F1: Fn(EchoRequest) -> Fut1 + Sync + Send + 'static,
    Fut1: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
{
    fn descriptor(&self) -> ServiceDescriptor {
        ServiceDescriptor {
            protocol: Box::new(Nshead::default()),
            full_name: "nshead",
        }
    }

    async fn call_method(&self, _method: &str, req: &[u8]) -> server_kit::Result<Vec<u8>> {
        let mut echo_req = EchoRequest::new();
        echo_req.merge_from_bytes(req).unwrap();

        let echo_resp = self.echo(echo_req).await?;

        let echo_resp = echo_resp.write_to_bytes().unwrap();
        Ok(echo_resp)
    }
}
