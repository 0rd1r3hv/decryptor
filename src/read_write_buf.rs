use std::io::{self, BufReader, BufWriter, Read, Write};
use std::fs::File;

pub struct ReadWriteBuf {
    reader: BufReader<File>,
    writer: BufWriter<File>,
    buffer: Vec<u8>,
    file_size: usize,
    position: usize,
}

impl ReadWriteBuf {
    pub fn new(input_path: &str, output_path: &str, buf_size: usize) -> io::Result<Self> {
        let input = File::open(input_path)?;
        let output = File::create(output_path)?;
        let file_size = input.metadata()?.len() as usize;

        Ok(Self {
            reader: BufReader::new(input),
            writer: BufWriter::new(output),
            buffer: vec![0; buf_size],
            file_size,
            position: 0,
        })

    }

    pub fn process_with<F>(&mut self, size: usize, mut processor: F) -> io::Result<usize>
    where
        F: FnMut(&mut [u8], usize, usize),
    {
        let read_size = self.reader.read(&mut self.buffer[..size])?;
        processor(&mut self.buffer, self.position, read_size);
        self.writer.write_all(&self.buffer[..read_size])?;
        self.position += read_size;
        Ok(read_size)
    }
    
    pub fn get_position(&self) -> usize {
        self.position
    }
    
    pub fn get_file_size(&self) -> usize {
        self.file_size
    }

}

impl Drop for ReadWriteBuf {
    fn drop(&mut self) {
        let _ = self.writer.flush();
    }
}