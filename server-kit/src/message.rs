pub struct Message {
    buf: Vec<u8>,
}

impl Message {
    pub fn new(buf: Vec<u8>) -> Self {
        Self { buf }
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.buf
    }
}
