use async_trait::async_trait;
use tokio::net::TcpStream;

use crate::{Message, Result};

pub mod nshead;

#[async_trait]
pub trait Protocol {
    // for server and channel
    async fn parse(&self, stream: &mut TcpStream) -> Result<Vec<u8>>;

    // for server
    fn process_request(&self, _buf: Vec<u8>) -> Result<Message> {
        unimplemented!()
    }

    fn pack_response(&self, _msg: Message) -> Vec<u8> {
        unimplemented!()
    }

    // for channel
    fn pack_request(&self, _msg: Message) -> Vec<u8> {
        unimplemented!()
    }

    fn process_response(&self, _buf: Vec<u8>) -> Result<Message> {
        unimplemented!()
    }
}
