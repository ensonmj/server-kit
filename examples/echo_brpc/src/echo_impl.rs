use std::future::Future;

use async_trait::async_trait;
use protobuf::Message;
use tracing::instrument;

use server_kit::{Result, Service, ServiceDescriptor};

use crate::{
    echo::{EchoRequest, EchoResponse},
    EchoService, EchoSeviceDescriptor,
};

pub struct EchoServiceImpl<F1, Fut1, F2, Fut2>
where
    F1: Fn(EchoRequest) -> Fut1 + Sync + Send + 'static,
    Fut1: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
    F2: Fn(EchoRequest) -> Fut2 + Sync + Send + 'static,
    Fut2: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
{
    echo_fn: F1,
    another_echo_fn: F2,
}

impl<F1, Fut1, F2, Fut2> EchoServiceImpl<F1, Fut1, F2, Fut2>
where
    F1: Fn(EchoRequest) -> Fut1 + Sync + Send + 'static,
    Fut1: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
    F2: Fn(EchoRequest) -> Fut2 + Sync + Send + 'static,
    Fut2: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
{
    pub fn new(echo_fn: F1, another_echo_fn: F2) -> Self {
        Self {
            echo_fn,
            another_echo_fn,
        }
    }
}

#[async_trait]
impl<F1, Fut1, F2, Fut2> Service for EchoServiceImpl<F1, Fut1, F2, Fut2>
where
    F1: Fn(EchoRequest) -> Fut1 + Sync + Send + 'static,
    Fut1: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
    F2: Fn(EchoRequest) -> Fut2 + Sync + Send + 'static,
    Fut2: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
{
    fn descriptor(&self) -> &dyn ServiceDescriptor {
        &EchoSeviceDescriptor {}
    }

    async fn call_method(&self, method: &str, req: &[u8]) -> server_kit::Result<Vec<u8>> {
        let mut echo_req = EchoRequest::new();
        echo_req.merge_from_bytes(req).unwrap();

        let echo_resp = match method {
            "echo" => self.echo(echo_req).await,
            "another_echo" => self.another_echo(echo_req).await,
            _ => Err(server_kit::Error::StrErr(format!(
                "cant't find method[{method}]"
            ))),
        }
        .unwrap();

        let echo_resp = echo_resp.write_to_bytes().unwrap();
        Ok(echo_resp)
    }
}

#[async_trait]
impl<F1, Fut1, F2, Fut2> EchoService for EchoServiceImpl<F1, Fut1, F2, Fut2>
where
    F1: Fn(EchoRequest) -> Fut1 + Sync + Send + 'static,
    Fut1: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
    F2: Fn(EchoRequest) -> Fut2 + Sync + Send + 'static,
    Fut2: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
{
    #[instrument(skip_all)]
    async fn echo(&self, buf: EchoRequest) -> server_kit::Result<EchoResponse> {
        (self.echo_fn)(buf).await
    }

    #[instrument(skip_all)]
    async fn another_echo(&self, buf: EchoRequest) -> server_kit::Result<EchoResponse> {
        (self.another_echo_fn)(buf).await
    }
}
