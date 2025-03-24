use std::fs::File;
use std::io::{BufReader, BufWriter};
use crate::decryptor::Decryptor;

const DEFAULT_SEGMENT_SIZE: usize = 4096;

impl Decryptor {
    fn new(input_path: String, output_path: String) -> io::Result<Self> {
        let input_file = File::open(input_path)?;
        let output_file = File::create(output_path)?;
        Ok(Self {
            reader: BufReader::new(input_file)?,
            writer: BufWriter::new(output_file)?,
            seg_size: DEFAULT_SEGMENT_SIZE,
        })
    }

    fn read(&self) -> io::Result<&[u8]> {
        self.reader.fill_buf()
    }

    fn write(&self, data: &[u8]) -> io::Result<usize> {
    }
}