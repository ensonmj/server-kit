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

#[derive(Default)]
pub struct ServiceManger {
    services: HashMap<TypeId, Box<dyn Protocol>>,
}

impl ServiceManger {
    pub fn add_service<P, S>(&mut self, svc: S) -> Result<()>
    where
        P: Protocol,
        S: Service,
    {
        let type_id = P::protocol_id();
        let svc_name = svc.descriptor().full_name();

        let protocol = self
            .services
            .entry(type_id)
            .or_insert_with(|| Box::new(P::default()));
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
