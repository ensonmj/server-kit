use std::{any::TypeId, collections::HashMap, future::Future};

use async_trait::async_trait;
use protobuf::Message;
use tokio::net::TcpStream;
use tracing::instrument;

use server_kit::{
    message::CommonMsg,
    protocol::{Nshead, Protocol},
    Result, Service, ServiceDescriptor,
};

use crate::{
    echo::{EchoRequest, EchoResponse},
    EchoService, EchoSeviceDescriptor,
};

pub struct EchoServiceImpl<F1, Fut1>
where
    F1: Fn(EchoRequest) -> Fut1 + Sync + Send + 'static,
    Fut1: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
{
    protocol: Box<dyn Protocol>,
    echo_fn: F1,
}

impl<F1, Fut1> EchoServiceImpl<F1, Fut1>
where
    F1: Fn(EchoRequest) -> Fut1 + Sync + Send + 'static,
    Fut1: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
{
    pub fn new(echo_fn: F1) -> Self {
        Self {
            protocol: Box::new(Nshead),
            echo_fn,
        }
    }
}

#[async_trait]
impl<F1, Fut1> Service for EchoServiceImpl<F1, Fut1>
where
    F1: Fn(EchoRequest) -> Fut1 + Sync + Send + 'static,
    Fut1: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
{
    fn descriptor(&self) -> &dyn ServiceDescriptor {
        &EchoSeviceDescriptor {}
    }

    async fn call_method(&self, _method: &str, req: &[u8]) -> server_kit::Result<Vec<u8>> {
        let mut echo_req = EchoRequest::new();
        echo_req.merge_from_bytes(req).unwrap();

        let echo_resp = self.echo(echo_req).await?;

        let echo_resp = echo_resp.write_to_bytes().unwrap();
        Ok(echo_resp)
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
impl<F1, Fut1> Protocol for EchoServiceImpl<F1, Fut1>
where
    F1: Fn(EchoRequest) -> Fut1 + Sync + Send + 'static,
    Fut1: Future<Output = Result<EchoResponse>> + Sync + Send + 'static,
{
    fn protocol_id(&self) -> TypeId {
        self.protocol.protocol_id()
    }

    // for server and channel
    async fn parse(&self, stream: &mut TcpStream) -> Result<CommonMsg> {
        self.protocol.parse(stream).await
    }

    // for server
    async fn process_request(
        &self,
        msg: CommonMsg,
        services: &HashMap<&'static str, Box<dyn Service>>,
    ) -> Result<CommonMsg> {
        self.protocol.process_request(msg, services).await
    }
    fn pack_response(&self, msg: CommonMsg) -> Vec<u8> {
        self.protocol.pack_request(msg)
    }

    // for channel
    fn pack_request(&self, msg: CommonMsg) -> Vec<u8> {
        self.protocol.pack_request(msg)
    }
    async fn process_response(&self, msg: CommonMsg) -> Result<Vec<u8>> {
        self.protocol.process_response(msg).await
    }
}