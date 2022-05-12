use async_trait::async_trait;
use protobuf::{Message, MessageField};
use tracing::{debug, instrument};

use server_kit::{
    channel::Channel, message::CommonMsg, protocol::Protocol, Result, Service, ServiceDescriptor,
};
use server_kit_protocol::baidu_rpc_meta::{RpcMeta, RpcRequestMeta};

use crate::{
    echo::{EchoRequest, EchoResponse},
    EchoService, EchoSeviceDescriptor,
};

pub struct EchoStub<P>
where
    P: Protocol + Sync + Send + 'static,
{
    channel: Channel<P>,
}

impl<P> EchoStub<P>
where
    P: Protocol + Sync + Send + 'static,
{
    pub fn new(channel: Channel<P>) -> Self {
        Self { channel }
    }
}

#[async_trait]
impl<P> EchoService for EchoStub<P>
where
    P: Protocol + Sync + Send + 'static,
{
    #[instrument(skip_all)]
    async fn echo(&self, req: EchoRequest) -> Result<EchoResponse> {
        let mut meta = RpcMeta::new();
        let mut req_meta = RpcRequestMeta::new();
        let svc_name = EchoSeviceDescriptor {}.full_name().to_string();
        let method_name = "echo".to_string();
        debug!("set service name[{svc_name}], method name[{method_name}]");
        req_meta.set_service_name(svc_name);
        req_meta.set_method_name(method_name);
        meta.request = MessageField::some(req_meta);

        let req = req.write_to_bytes()?;
        let mut msg = CommonMsg::new(req);
        msg.with_meta(meta.write_to_bytes()?);

        let resp = self.channel.process(msg).await?;

        Ok(EchoResponse::parse_from_bytes(&resp)?)
    }
}

#[async_trait]
impl<P> Service for EchoStub<P>
where
    P: Protocol + Sync + Send + 'static,
{
    fn descriptor(&self) -> &dyn ServiceDescriptor {
        unimplemented!()
    }

    async fn call_method(&self, _method: &str, _req: &[u8]) -> server_kit::Result<Vec<u8>> {
        unimplemented!()
    }
}
