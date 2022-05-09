use bytes::BufMut;
use bytes::BytesMut;
use tracing::instrument;
use tracing::{debug, warn};

use super::Protocol;
use crate::Error;
use crate::Result;

pub const NSHEAD_MAGICNUM: u32 = 0xfb709394;
pub const NSHEAD_LEN: usize = ::std::mem::size_of::<Nshead>();

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

impl Protocol for Nshead {
    #[instrument(skip_all)]
    fn parse<'buf>(&self, buf: &'buf [u8]) -> Result<&'buf [u8]> {
        if buf.len() < NSHEAD_LEN {
            return Err(Error::Err("len too small".to_string()));
        }
        let data = &buf[..NSHEAD_LEN].try_into().unwrap();
        let head = Self::from_u8_slice(data);
        if head.magic_num != NSHEAD_MAGICNUM {
            warn!("Unexpected header: {:?}", head);
            return Err(Error::MagicNum(format!(
                "unexpected header magic_num[{}]",
                head.magic_num
            )));
        }
        debug!("Receive header: {:?}", head);

        let msg_len = NSHEAD_LEN + head.body_len as usize;

        if buf.len() < msg_len {
            return Err(Error::Err("len too small".to_string()));
        }

        Ok(&buf[NSHEAD_LEN..msg_len])
    }

    #[instrument(skip_all)]
    fn process_request(&self, buf: &[u8]) -> Result<Vec<u8>> {
        debug!("receive message");
        Ok(buf.to_vec())
    }

    #[instrument(skip_all)]
    fn pack_response(&self, buf: &[u8]) -> Vec<u8> {
        let mut buffer = BytesMut::with_capacity(NSHEAD_LEN + buf.len());
        let head = Self::default_with_len(buf.len() as u32);
        buffer.put(head.as_u8_slice());
        buffer.put(buf);

        buffer.to_vec()
    }

    #[instrument(skip_all)]
    fn process_response(&self, buf: &[u8]) -> Result<Vec<u8>> {
        Ok(buf.to_vec())
    }

    #[instrument(skip_all)]
    fn pack_request(&self, buf: &[u8]) -> Vec<u8> {
        let mut buffer = BytesMut::with_capacity(NSHEAD_LEN + buf.len());
        let head = Self::default_with_len(buf.len() as u32);
        buffer.put(head.as_u8_slice());
        buffer.put(buf);

        buffer.to_vec()
    }
}
