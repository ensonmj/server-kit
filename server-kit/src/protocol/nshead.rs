use std::cmp::Ordering;

use async_trait::async_trait;
use bytes::BufMut;
use bytes::BytesMut;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::instrument;
use tracing::{debug, warn};

use super::Protocol;
use crate::error::ParseError;
use crate::Error;
use crate::Message;
use crate::Result;

pub const NSHEAD_MAGICNUM: u32 = 0xfb709394;
pub const NSHEAD_LEN: usize = ::std::mem::size_of::<Nshead>();
const BUF_SIZE: usize = 4096;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Nshead {
    id: u16,
    version: u16,
    log_id: u32,
    provider: [u8; 16],
    pub magic_num: u32,
    reserved: u32,
    pub body_len: u32,
}

impl Default for Nshead {
    fn default() -> Self {
        Self {
            id: Default::default(),
            version: Default::default(), // maybe 778
            log_id: Default::default(),
            provider: Default::default(),
            magic_num: NSHEAD_MAGICNUM,
            reserved: Default::default(),
            body_len: Default::default(),
        }
    }
}

impl Nshead {
    pub fn default_with_len(body_len: u32) -> Self {
        Self {
            body_len,
            ..Default::default()
        }
    }

    pub fn from_u8_slice(bytes: &[u8; ::std::mem::size_of::<Self>()]) -> Self {
        unsafe { std::mem::transmute(*bytes) }
    }

    pub fn as_u8_slice(&self) -> &[u8] {
        unsafe {
            ::std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                ::std::mem::size_of::<Self>(),
            )
        }
    }
}

#[async_trait]
impl Protocol for Nshead {
    #[instrument(skip_all)]
    async fn parse(stream: &mut TcpStream) -> Result<Vec<u8>> {
        let mut buf = BytesMut::with_capacity(BUF_SIZE);
        loop {
            if stream.read_buf(&mut buf).await? == 0 {
                return Err(Error::Parse(ParseError::UnexpectedEof));
            }
            debug!("read from stream: {buf:?}");

            if buf.len() >= NSHEAD_LEN {
                break;
            }
        }
        let nshead = &buf[..NSHEAD_LEN].try_into().unwrap();
        let nshead = Self::from_u8_slice(nshead);
        if nshead.magic_num != NSHEAD_MAGICNUM {
            warn!("unexpected header: {:?}", nshead);
            return Err(Error::Parse(ParseError::TryOther));
        }
        debug!("receive header: {:?}", nshead);

        let _ = buf.split_to(NSHEAD_LEN);
        loop {
            match nshead.body_len.cmp(&(buf.len() as u32)) {
                Ordering::Equal => break,
                Ordering::Greater => {
                    if stream.read_buf(&mut buf).await? == 0 {
                        return Err(Error::Parse(ParseError::UnexpectedEof));
                    }
                    debug!("read from stream: {buf:?}");
                }
                Ordering::Less => unimplemented!(),
            };
        }

        Ok(buf.to_vec())
    }

    #[instrument(skip_all)]
    fn process_request(buf: Vec<u8>) -> Result<Message> {
        debug!("receive request");
        Ok(Message::new(buf.to_vec()))
    }

    #[instrument(skip_all)]
    fn pack_response(msg: Message) -> Vec<u8> {
        let msg = msg.to_vec();

        let mut nshead = Self::default_with_len(msg.len() as u32);
        nshead.body_len = msg.len() as u32;

        let mut buffer = BytesMut::with_capacity(NSHEAD_LEN + msg.len());
        buffer.put(nshead.as_u8_slice());
        buffer.put(msg.as_slice());

        buffer.to_vec()
    }

    #[instrument(skip_all)]
    fn process_response(buf: Vec<u8>) -> Result<Message> {
        debug!("receive response");
        Ok(Message::new(buf.to_vec()))
    }

    #[instrument(skip_all)]
    fn pack_request(msg: Message) -> Vec<u8> {
        let msg = msg.to_vec();

        let mut nshead = Self::default_with_len(msg.len() as u32);
        nshead.body_len = msg.len() as u32;

        let mut buffer = BytesMut::with_capacity(NSHEAD_LEN + msg.len());
        buffer.put(nshead.as_u8_slice());
        buffer.put(msg.as_slice());

        buffer.to_vec()
    }
}
