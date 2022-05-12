use bytes::{BufMut, BytesMut};

#[derive(Default, Debug)]
pub struct CommonMsg {
    pub meta: Vec<u8>,
    pub payload: Vec<u8>,
}

impl CommonMsg {
    pub fn new(payload: Vec<u8>) -> Self {
        Self {
            meta: vec![],
            payload,
        }
    }

    pub fn with_meta(&mut self, meta: Vec<u8>) {
        self.meta = meta;
    }

    pub fn with_payload(&mut self, payload: Vec<u8>) {
        self.payload = payload;
    }

    pub fn to_vec(self) -> Vec<u8> {
        match self.meta.len() {
            0 => self.payload,
            meta_len => {
                let len = meta_len + self.payload.len();
                let mut buf = BytesMut::with_capacity(len);
                buf.put(self.meta.as_slice());
                buf.put(self.payload.as_slice());
                buf.to_vec()
            }
        }
    }

    pub fn meta_size(&self) -> u32 {
        self.meta.len() as u32
    }

    pub fn payload_size(&self) -> u32 {
        self.payload.len() as u32
    }

    pub fn body_size(&self) -> u32 {
        self.meta_size() + self.payload_size()
    }
}
