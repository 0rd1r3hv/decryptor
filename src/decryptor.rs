use std::io::{self, BufReader, BufWriter, Read, Write, Seek, SeekFrom};
use std::fs::File;
use std::process::Command;
use md5;
use byteorder::{LittleEndian, ReadBytesExt};
use aes::Aes128;
use aes::cipher::{BlockDecryptMut, KeyIvInit,generic_array::GenericArray};

mod decrypt;
mod key;
mod tweaked_tea;
pub struct Decryptor {
    reader: BufReader<File>,
    writer: BufWriter<File>,
    block: Vec<u8>,
    decrypt_key: DecryptKey,
    decrypted_db: Vec<u8>,
}

pub struct DecryptKey {
    key: Vec<u8>,
    hash: u32,
    sbox: Vec<u8>,
    key_len: usize,
}

impl Decryptor {
    pub fn new(input_path: &str, output_path: &str, block_size: usize, key: &str) -> io::Result<Self> {
        let input_file = File::open(input_path)?;
        let output_file = File::create(output_path)?;

        Ok(Self {
            reader: BufReader::new(input_file),
            writer: BufWriter::new(output_file),
            block: vec![0; block_size],
            decrypt_key: DecryptKey::new(key),
            decrypted_db: Self::dec_db(Self::gen_db_key())?,
        })
    }

    fn read(&mut self) -> io::Result<usize> {
        self.reader.read(&mut self.block)
    }

    fn write(&mut self, size: usize) -> io::Result<()> {
        self.writer.write_all(&self.block[..size])
    }

    fn get_current_position(&mut self) -> io::Result<u64> {
        self.writer.stream_position()
    }

    fn gen_db_key() -> [u8; 16] {
        let suffix: Vec<u8> = vec![0x5C, 0xBD, 0x98, 0x7C, 0x1C, 0x38, 0x17, 0x8E];
        let db_key = Command::new("powershell")
            .arg("-File")
            .arg(".\\scripts\\keygen.ps1")
            .output()
            .expect("Fail to execute PowerShell script")
            .stdout;
        let db_key = md5::compute([&db_key[..db_key.len() - 2], &suffix].concat());
        format!(
            "{:08X}{:02X}{:02X}",
            u32::from_le_bytes((&db_key[0..4]).try_into().unwrap()),
            u16::from_le_bytes((&db_key[4..6]).try_into().unwrap()),
            u16::from_le_bytes((&db_key[6..8]).try_into().unwrap()))
            .as_bytes()
            .try_into()
            .expect("Key length error")
    }

    fn dec_db(db_key: [u8; 16]) -> io::Result<Vec<u8>> {
        let app_data = std::env::var("APPDATA").unwrap_or_default();
        let mut db = File::open(format!("{}\\Tencent\\QQMusic\\Driveredbb.dat", app_data))?;
        let mut crc = File::open(format!("{}\\Tencent\\QQMusic\\Driveredbb.dat.crc", app_data))?;
        let size = db.read_u32::<LittleEndian>()? as usize;
        let buf_size = match size % 16 {
            0 => size,
            _ => ((size >> 4) + 1) << 4,
        };

        let mut db_content = vec![0; buf_size];
        let mut iv = [0u8; 16];

        db.read_exact(&mut db_content)?;
        crc.seek(SeekFrom::Current(12))?;
        crc.read_exact(&mut iv)?;

        let mut cipher = cfb_mode::Decryptor::<Aes128>::new(&db_key.into(), &iv.into());
        let mut blocks: Vec<_> = db_content.chunks_exact(16)
            .map(GenericArray::clone_from_slice)
            .collect();
        cipher.decrypt_blocks_mut(&mut blocks);
        Ok(blocks.into_iter().flatten().collect::<Vec<u8>>()[4..size].to_vec())
    }

}

impl Drop for Decryptor {
    fn drop(&mut self) {
        let _ = self.writer.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dec_db() {
        let result = Decryptor::dec_db(Decryptor::gen_db_key())
            .expect("Failed to decrypt db");
        println!("{:X}", result.len());
        println!("{}", String::from_utf8_lossy(&result));
    }
}
