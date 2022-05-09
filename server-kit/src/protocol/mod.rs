use crate::Result;

pub mod nshead;

pub trait Protocol {
    // for server and channel
    fn parse<'buf>(&self, buf: &'buf [u8]) -> Result<&'buf [u8]>;

    // for server
    fn process_request(&self, _buf: &[u8]) -> Result<Vec<u8>> {
        unimplemented!()
    }
    fn pack_response(&self, _buf: &[u8]) -> Vec<u8> {
        unimplemented!()
    }

    // for channel
    fn pack_request(&self, _buf: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
    fn process_response(&self, _buf: &[u8]) -> Result<Vec<u8>> {
        unimplemented!()
    }
}
