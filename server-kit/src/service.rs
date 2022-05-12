use std::any::TypeId;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use async_trait::async_trait;
use tracing::debug;
use tracing::instrument;

use crate::error::ParseErr;
use crate::error::SvcErr;
use crate::protocol::Protocol;
use crate::Error;
use crate::Result;

pub trait ServiceDescriptor {
    // The name of the service, not including its containing scope.
    fn name(&self) -> &'static str;
    // The fully-qualified name of the service, scope delimited by periods.
    fn full_name(&self) -> &'static str;
}

#[async_trait]
pub trait Service: Sync + Send + 'static {
    fn descriptor(&self) -> &dyn ServiceDescriptor;
    async fn call_method(&self, method_name: &str, req: &[u8]) -> Result<Vec<u8>>;
}

struct ProtocolServices {
    protocol: Box<dyn Protocol>,
    services: HashMap<&'static str, Box<dyn Service>>,
}

#[derive(Default)]
pub struct ServiceManger {
    services: HashMap<TypeId, ProtocolServices>,
}

impl ServiceManger {
    pub fn add_service<P>(&mut self, protocol: P, svc: Box<dyn Service>) -> Result<()>
    where
        P: Protocol,
    {
        let type_id = protocol.protocol_id();
        let svc_name = svc.descriptor().full_name();

        let protocol_svcs = self
            .services
            .entry(type_id)
            .or_insert_with(|| ProtocolServices {
                services: HashMap::new(),
                protocol: Box::new(protocol),
            });
        if protocol_svcs.services.contains_key(svc_name) {
            return Err(SvcErr::Exist(svc_name.to_string()).into());
        }
        protocol_svcs.services.insert(svc_name, svc);

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn process(&self, stream: &mut TcpStream) -> Result<()> {
        for protocol_svcs in self.services.values() {
            let protocol = &protocol_svcs.protocol;
            let svcs = &protocol_svcs.services;

            let msg = match protocol.parse(stream).await {
                Ok(msg) => msg,
                Err(Error::Parse(ParseErr::TryOther)) => continue,
                Err(err) => return Err(err),
            };

            // parse request
            let msg = protocol.process_request(msg, svcs).await?;
            let msg = protocol.pack_response(msg);

            // write response
            debug!("write message: {msg:?}");
            stream.write_all(&msg).await?;
        }

        Ok(())
    }
}
