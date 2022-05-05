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
