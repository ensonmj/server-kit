use crate::Result;

pub mod nshead;

pub trait Protocol {
    fn parse<'buf>(&self, buf: &'buf [u8]) -> Result<&'buf [u8]>;
    fn process_request(&self, buf: &[u8]) -> Result<Vec<u8>>;
    fn pack_response(&self, buf: &[u8]) -> Vec<u8>;
}
