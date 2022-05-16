use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use async_trait::async_trait;
use tracing::debug;
use tracing::instrument;

use crate::error::ParseErr;
use crate::protocol::Protocol;
use crate::Error;
use crate::Result;

pub struct ServiceDescriptor {
    pub protocol: Box<dyn Protocol>,
    // The name of the service, not including its containing scope.
    // fn name(&self) -> &'static str;
    // The fully-qualified name of the service, scope delimited by periods.
    // fn full_name(&self) -> &'static str;
    pub full_name: &'static str,
}

#[async_trait]
pub trait Service: Sync + Send + 'static {
    fn descriptor(&self) -> ServiceDescriptor
    where
        Self: Sized;
    async fn call_method(&self, method_name: &str, req: &[u8]) -> Result<Vec<u8>>;
}

#[derive(Default)]
pub struct ServiceManger {
    services: HashMap<TypeId, Box<dyn Protocol>>,
}

impl ServiceManger {
    pub fn add_service<S>(&mut self, svc: S) -> Result<()>
    where
        S: Service,
    {
        let svc_desc = svc.descriptor();
        let type_id = svc_desc.protocol.type_id();
        let protocol = self
            .services
            .entry(type_id)
            .or_insert_with(|| svc_desc.protocol);

        let svc_name = svc_desc.full_name;
        protocol.add_service(svc_name.to_string(), Box::new(svc))
    }

    #[instrument(skip_all)]
    pub async fn process(&self, stream: &mut TcpStream) -> Result<()> {
        for protocol in self.services.values() {
            let msg = match protocol.parse(stream).await {
                Ok(msg) => msg,
                Err(Error::Parse(ParseErr::TryOther)) => continue,
                Err(err) => return Err(err),
            };

            // parse request
            let msg = protocol.process_request(msg).await?;
            let msg = protocol.pack_response(msg);

            // write response
            debug!("write message: {msg:?}");
            stream.write_all(&msg).await?;
        }

        Ok(())
    }
}
