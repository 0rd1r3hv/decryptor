pub const BLOCK_SIZE: usize = 0x1400;
pub const HEADER_SIZE: usize = 0x80;
pub const KEY_LEN_MASK: usize = 0x1FF;
pub const MAGIC: [u8; 8] = [0x69, 0x56, 0x46, 0x38, 0x2B, 0x20, 0x15, 0x0B];
pub const TAIL_SIZE: usize = 0xC0;
pub const HEX_CAHRS: [u8; 16] = *b"0123456789abcdef";
