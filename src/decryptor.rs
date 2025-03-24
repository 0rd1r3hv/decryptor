use std::io::{self,BufReader, BufWriter, Read, Write, Seek};
use std::fs::File;

mod decrypt;
mod key;
pub struct Decryptor {
    reader: BufReader<File>,
    writer: BufWriter<File>,
    segment: Vec<u8>,
    decrypt_key: DecryptKey,
}

pub struct DecryptKey {
    key: Vec<u8>,
    hash: u32,
    sbox: Vec<u8>,
    key_len: usize,
}

impl Decryptor {
    pub fn new(input_path: &str, output_path: &str, seg_size: usize, key: &str) -> io::Result<Self> {
        let input_file = File::open(input_path)?;
        let output_file = File::create(output_path)?;
        Ok(Self {
            reader: BufReader::new(input_file),
            writer: BufWriter::new(output_file),
            segment: vec![0; seg_size],
            decrypt_key: DecryptKey::new(key),
        })
    }

    fn read(&mut self) -> io::Result<usize> {
        self.reader.read(&mut self.segment)
    }

    fn write(&mut self, size: usize) -> io::Result<()> {
        self.writer.write_all(&self.segment[..size])
    }

    fn get_current_position(&mut self) -> io::Result<u64> {
        self.writer.stream_position()
    }
}

impl Drop for Decryptor {
    fn drop(&mut self) {
        let _ = self.writer.flush();
    }
}