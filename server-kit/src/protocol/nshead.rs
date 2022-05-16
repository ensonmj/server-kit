use std::any::TypeId;
use std::cmp::Ordering;
use std::fmt;

use async_trait::async_trait;
use bytes::BufMut;
use bytes::BytesMut;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::instrument;
use tracing::{debug, warn};

use super::Protocol;
use crate::error::ParseErr;
use crate::error::SvcErr;
use crate::global::BUF_SIZE;
use crate::message::CommonMsg;
use crate::Result;
use crate::Service;

pub const NSHEAD_MAGICNUM: u32 = 0xfb709394;
pub const NSHEAD_SIZE: usize = ::std::mem::size_of::<Header>();

pub struct Nshead(Option<(String, Box<dyn Service>)>);

#[async_trait]
impl Protocol for Nshead {
    fn default() -> Self {
        Nshead(None)
    }
    fn protocol_id() -> TypeId {
        TypeId::of::<Self>()
    }

    fn add_service(&mut self, svc_name: String, svc: Box<dyn Service>) -> Result<()> {
        match &self.0 {
            Some((svc_name, _)) => return Err(SvcErr::Exist(svc_name.clone()).into()),
            None => self.0 = Some((svc_name, svc)),
        }

        Ok(())
    }

    #[instrument(skip_all)]
    async fn parse(&self, stream: &mut TcpStream) -> Result<CommonMsg> {
        let mut buf = BytesMut::with_capacity(BUF_SIZE);
        loop {
            if stream.read_buf(&mut buf).await? == 0 {
                return Err(ParseErr::UnexpectedEof.into());
            }
            debug!("read from stream: {buf:?}");

            if buf.len() >= NSHEAD_SIZE {
                break;
            }
        }
        let head = &buf[..NSHEAD_SIZE].try_into().unwrap();
        let head = Header::from_u8_slice(head);
        if head.magic_num != NSHEAD_MAGICNUM {
            warn!(%head, "unexpected header");
            return Err(ParseErr::TryOther.into());
        }
        debug!(%head, "finish to parse nshead");

        let _ = buf.split_to(NSHEAD_SIZE);
        loop {
            match head.body_size.cmp(&(buf.len() as u32)) {
                Ordering::Equal => break,
                Ordering::Greater => {
                    if stream.read_buf(&mut buf).await? == 0 {
                        return Err(ParseErr::UnexpectedEof.into());
                    }
                    debug!("read from stream: {buf:?}");
                }
                Ordering::Less => unimplemented!(),
            };
        }

        Ok(CommonMsg::new(buf.to_vec()))
    }

    #[instrument(skip_all)]
    async fn process_request(
        &self,
        msg: CommonMsg,
        // services: &HashMap<&'static str, Box<dyn Service>>,
    ) -> Result<CommonMsg> {
        // let (_, svc) = services
        //     .iter()
        //     .next()
        //     .ok_or_else(|| SvcErr::NotExist("nshead".to_string()))?;
        let (_, svc) = self.0.as_ref().unwrap();
        let msg = svc.call_method("", &msg.payload).await?;
        let msg = CommonMsg::new(msg);
        Ok(msg)
    }

    #[instrument(skip_all)]
    fn pack_response(&self, msg: CommonMsg) -> Vec<u8> {
        let msg = msg.to_vec();

        let mut head = Header::default_with_len(msg.len() as u32);
        head.body_size = msg.len() as u32;

        let mut buffer = BytesMut::with_capacity(NSHEAD_SIZE + msg.len());
        buffer.put(head.as_u8_slice());
        buffer.put(msg.as_slice());

        buffer.to_vec()
    }

    #[instrument(skip_all)]
    async fn process_response(&self, msg: CommonMsg) -> Result<Vec<u8>> {
        Ok(msg.payload)
    }

    #[instrument(skip_all)]
    fn pack_request(&self, msg: CommonMsg) -> Vec<u8> {
        let msg = msg.to_vec();

        let mut nshead = Header::default_with_len(msg.len() as u32);
        nshead.body_size = msg.len() as u32;

        let mut buffer = BytesMut::with_capacity(NSHEAD_SIZE + msg.len());
        buffer.put(nshead.as_u8_slice());
        buffer.put(msg.as_slice());

        buffer.to_vec()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Header {
    id: u16,
    version: u16,
    log_id: u32,
    provider: [u8; 16],
    pub magic_num: u32,
    reserved: u32,
    pub body_size: u32,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            id: Default::default(),
            version: Default::default(), // maybe 778
            log_id: Default::default(),
            provider: Default::default(),
            magic_num: NSHEAD_MAGICNUM,
            reserved: Default::default(),
            body_size: Default::default(),
        }
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}:{:?}:{}:{}",
            self.id, self.version, self.log_id, self.provider, self.reserved, self.body_size
        )
    }
}

impl Header {
    pub fn default_with_len(body_len: u32) -> Self {
        Self {
            body_size: body_len,
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
