use std::any::TypeId;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use protobuf::Message;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::instrument;
use tracing::{debug, warn};

use server_kit_protocol::baidu_rpc_meta::RpcMeta;

use super::Protocol;
use crate::error::{ParseErr, SvcErr};
use crate::global::BUF_SIZE;
use crate::message::CommonMsg;
use crate::{Result, Service};

const HEADER_SIZE: usize = ::std::mem::size_of::<Header>();
const TAG: [u8; 4] = *b"PRPC";
const TAG_SIZE: usize = 4;
const BODY_START: usize = TAG_SIZE;
const BODY_SIZE: usize = 4;
const META_START: usize = BODY_START + BODY_SIZE;

pub struct Brpc(HashMap<String, Box<dyn Service>>);

#[async_trait]
impl Protocol for Brpc {
    fn default() -> Self {
        Brpc(HashMap::default())
    }
    fn protocol_id() -> TypeId {
        TypeId::of::<Self>()
    }

    fn add_service(&mut self, svc_name: String, svc: Box<dyn Service>) -> Result<()> {
        let services = &mut self.0;
        if services.contains_key(&svc_name) {
            return Err(SvcErr::Exist(svc_name).into());
        }
        services.insert(svc_name, svc);
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

            match buf.len() {
                n if n <= TAG_SIZE => {
                    if buf[..n] != TAG[..n] {
                        return Err(ParseErr::TryOther.into());
                    }
                    continue;
                }
                n if n > TAG_SIZE && n < HEADER_SIZE => {
                    if buf[..TAG_SIZE] != TAG[..] {
                        return Err(ParseErr::TryOther.into());
                    }
                    continue;
                }
                n if n >= HEADER_SIZE => {
                    if buf[..TAG_SIZE] != TAG[..] {
                        return Err(ParseErr::TryOther.into());
                    }
                    break;
                }
                _ => unimplemented!(),
            };
        }
        let head = buf[..HEADER_SIZE].try_into().unwrap();
        let head = Header::from_u8_slice(&head);
        if head.body_size < head.meta_size {
            warn!(%head, "body_size less than meta_size");
            return Err(ParseErr::TryOther.into());
        }
        debug!(%head, "finish to parse brpc header");

        let _ = buf.split_to(HEADER_SIZE);
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

        let mut msg = CommonMsg::default();
        msg.with_meta(buf.split_to(head.meta_size as usize).to_vec());
        msg.with_payload(buf.to_vec());

        Ok(msg)
    }

    #[instrument(skip_all)]
    async fn process_request(&self, msg: CommonMsg) -> crate::Result<CommonMsg> {
        // request
        let mut meta = RpcMeta::new();
        meta.merge_from_bytes(&msg.meta)?;
        let request_meta = meta.request;
        let svc_name = request_meta.service_name();

        let services = &self.0;
        if !services.contains_key(svc_name) {
            return Err(SvcErr::NotExist(svc_name.to_string()).into());
        }

        // process
        let svc = &services[svc_name];
        let method_name = request_meta.method_name();
        let msg = svc.call_method(method_name, &msg.payload).await?;

        // response
        let mut meta = RpcMeta::new();
        if let Some(resp) = meta.response.as_mut() {
            resp.set_error_code(0);
        }
        let meta = meta.write_to_bytes()?;
        let mut msg = CommonMsg::new(msg);
        msg.with_meta(meta);

        Ok(msg)
    }

    #[instrument(skip_all)]
    fn pack_response(&self, msg: CommonMsg) -> Vec<u8> {
        let mut buffer = BytesMut::with_capacity(HEADER_SIZE + msg.body_size() as usize);
        let head = Header::new(msg.payload_size(), msg.meta_size());
        buffer.put(head.as_u8_slice().as_slice());
        buffer.put(msg.to_vec().as_slice());

        buffer.to_vec()
    }

    #[instrument(skip_all)]
    async fn process_response(&self, msg: CommonMsg) -> crate::Result<Vec<u8>> {
        Ok(msg.payload)
    }

    #[instrument(skip_all)]
    fn pack_request(&self, msg: CommonMsg) -> Vec<u8> {
        let mut buffer = BytesMut::with_capacity(HEADER_SIZE + msg.body_size() as usize);
        let head = Header::new(msg.payload_size(), msg.meta_size());
        buffer.put(head.as_u8_slice().as_slice());
        buffer.put(msg.to_vec().as_slice());

        buffer.to_vec()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Header {
    tag: [u8; 4],   // "PRPC"
    body_size: u32, // network order, body_size include meta
    meta_size: u32, // network order
}

impl Default for Header {
    fn default() -> Self {
        Self {
            tag: TAG,
            body_size: 0,
            meta_size: 0,
        }
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.body_size, self.meta_size)
    }
}

impl Header {
    pub fn new(payload_size: u32, meta_size: u32) -> Self {
        Self {
            tag: TAG,
            body_size: meta_size + payload_size,
            meta_size,
        }
    }

    pub fn from_u8_slice(buf: &[u8; ::std::mem::size_of::<Self>()]) -> Self {
        Self {
            tag: TAG,
            body_size: u32::from_be_bytes(buf[BODY_START..META_START].try_into().unwrap()),
            meta_size: u32::from_be_bytes(buf[META_START..].try_into().unwrap()),
        }
    }

    pub fn as_u8_slice(&self) -> [u8; ::std::mem::size_of::<Self>()] {
        let mut buf = [0; ::std::mem::size_of::<Self>()];
        buf[..BODY_START].copy_from_slice(&TAG);
        buf[BODY_START..META_START].copy_from_slice(&self.body_size.to_be_bytes());
        buf[META_START..].copy_from_slice(&self.meta_size.to_be_bytes());
        buf
    }
}
