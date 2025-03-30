use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

pub struct ReadWriteBuf {
    reader: BufReader<File>,
    writer: BufWriter<File>,
    buffer: Vec<u8>,
    file_size: usize,
    position: usize,
}

impl ReadWriteBuf {
    pub fn new(
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
        buf_size: usize,
    ) -> io::Result<Self> {
        Ok(Self {
            file_size: input_path.as_ref().metadata()?.len() as usize,
            reader: BufReader::new(File::open(input_path)?),
            writer: BufWriter::new(File::create(output_path)?),
            buffer: vec![0; buf_size],
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

    pub const fn get_file_size(&self) -> usize {
        self.file_size
    }

    pub const fn get_position(&self) -> usize {
        self.position
    }
}

impl Drop for ReadWriteBuf {
    fn drop(&mut self) {
        let _ = self.writer.flush();
    }
}
