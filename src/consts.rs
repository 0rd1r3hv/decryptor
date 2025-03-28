pub const SKIP_TO_NAME: usize = 0x1;
pub const NAME_SIZE: usize = 0x18;
pub const SKIP_TO_KEY: usize = 0x4;
pub const KEY_SIZE: usize = 0x2C0;
pub const BLOCK_SIZE: usize = 0x1400;
pub const TAIL_SIZE: usize = 0xC0;
pub const KEY_LEN_MASK: usize = 0x1FF;
pub const HEADER_SIZE: usize = 0x80;
pub const MAGIC: [u8; 8] = [0x69, 0x56, 0x46, 0x38, 0x2B, 0x20, 0x15, 0x0B];