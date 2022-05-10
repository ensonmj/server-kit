use async_trait::async_trait;
use tokio::net::TcpStream;

use crate::{Message, Result};

pub mod nshead;

#[async_trait]
pub trait Protocol {
    // for server and channel
    async fn parse(stream: &mut TcpStream) -> Result<Vec<u8>>;

    // for server
    fn process_request(_buf: Vec<u8>) -> Result<Message> {
        unimplemented!()
    }

    fn pack_response(_msg: Message) -> Vec<u8> {
        unimplemented!()
    }

    // for channel
    fn pack_request(_msg: Message) -> Vec<u8> {
        unimplemented!()
    }

    fn process_response(_buf: Vec<u8>) -> Result<Message> {
        unimplemented!()
    }
}
