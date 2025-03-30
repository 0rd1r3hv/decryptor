use crate::utils::get_name_to_filename;
use aes::Aes128;
use aes::cipher::{BlockDecryptMut, KeyIvInit, generic_array::GenericArray};
use byteorder::{LittleEndian, ReadBytesExt};
use md5;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::process::Command;

mod decrypt;
mod decryption;

const SUFFIX: [u8; 8] = [0x5C, 0xBD, 0x98, 0x7C, 0x1C, 0x38, 0x17, 0x8E];
pub struct Decryptor {
    input_path: String,
    output_path: String,
    db_key: [u8; 16],
    decrypted_db: Vec<u8>,
    name_to_filename: HashMap<String, String>,
}

trait Cipher {
    fn decrypt(&self, buf: &mut [u8], cur_pos: usize, dec_size: usize);
}

pub struct Decryption {
    cipher: Box<dyn Cipher>,
}

impl Decryptor {
    pub fn new(input_path: &str, output_path: &str) -> io::Result<Self> {
        let input_path = input_path.to_string();
        let output_path = output_path.to_string();
        let db_key = Self::gen_db_key();
        let name_to_filename = get_name_to_filename(&input_path);
        let decrypted_db = Self::dec_db(db_key)?;
        Ok(Self {
            input_path,
            output_path,
            db_key,
            decrypted_db,
            name_to_filename,
        })
    }

    fn gen_db_key() -> [u8; 16] {
        let db_key = Command::new("powershell")
            .arg("-File")
            .arg(".\\scripts\\keygen.ps1")
            .output()
            .expect("Fail to execute PowerShell script")
            .stdout;
        let db_key = md5::compute([&db_key[..db_key.len() - 2], &SUFFIX].concat());
        format!(
            "{:08X}{:02X}{:02X}",
            u32::from_le_bytes((&db_key[0..4]).try_into().unwrap()),
            u16::from_le_bytes((&db_key[4..6]).try_into().unwrap()),
            u16::from_le_bytes((&db_key[6..8]).try_into().unwrap())
        )
        .as_bytes()
        .try_into()
        .expect("Key length error")
    }

    fn dec_db(db_key: [u8; 16]) -> io::Result<Vec<u8>> {
        let app_data = std::env::var("APPDATA").unwrap_or_default();
        let mut db = File::open(format!("{}\\Tencent\\QQMusic\\Driveredbb.dat", app_data))?;
        let mut crc = File::open(format!(
            "{}\\Tencent\\QQMusic\\Driveredbb.dat.crc",
            app_data
        ))?;
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
        let mut blocks: Vec<_> = db_content
            .chunks_exact(16)
            .map(GenericArray::clone_from_slice)
            .collect();
        cipher.decrypt_blocks_mut(&mut blocks);
        Ok(blocks.into_iter().flatten().collect::<Vec<u8>>()[4..size].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dec_db() {
        let result = Decryptor::dec_db(Decryptor::gen_db_key()).expect("Failed to decrypt db");
        let mut file = File::create("dump.bin").unwrap();
        file.write_all(&result).unwrap();
    }
}
