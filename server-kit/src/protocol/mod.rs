use std::any::TypeId;

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
    fn default() -> Self
    where
        Self: Sized;
    fn protocol_id() -> TypeId
    where
        Self: Sized;

    fn add_service(&mut self, svc_name: String, svc: Box<dyn Service>) -> Result<()>;

    // for server and channel
    async fn parse(&self, stream: &mut TcpStream) -> Result<CommonMsg>;

    // for server
    async fn process_request(&self, msg: CommonMsg) -> Result<CommonMsg>;
    fn pack_response(&self, msg: CommonMsg) -> Vec<u8>;

    // for channel
    fn pack_request(&self, msg: CommonMsg) -> Vec<u8>;
    async fn process_response(&self, msg: CommonMsg) -> Result<Vec<u8>>;
}
