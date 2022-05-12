use std::any::TypeId;
use std::collections::HashMap;

use async_trait::async_trait;
use tokio::net::TcpStream;

use crate::message::CommonMsg;
use crate::{Result, Service};

pub use brpc::Brpc;
pub use nshead::Nshead;

mod brpc;
mod nshead;

#[async_trait]
pub trait Protocol: Sync + Send + 'static {
    fn protocol_id(&self) -> TypeId;

    // for server and channel
    async fn parse(&self, stream: &mut TcpStream) -> Result<CommonMsg>;

    // for server
    async fn process_request(
        &self,
        msg: CommonMsg,
        services: &HashMap<&'static str, Box<dyn Service>>,
    ) -> Result<CommonMsg>;
    fn pack_response(&self, _msg: CommonMsg) -> Vec<u8>;

    // for channel
    fn pack_request(&self, _msg: CommonMsg) -> Vec<u8>;
    async fn process_response(&self, msg: CommonMsg) -> Result<Vec<u8>>;
}
