use crate::utils::{get_child_path_by_prfx_and_sfx, get_name_path_map};
use aes::Aes128;
use aes::cipher::{BlockDecryptMut, KeyIvInit, generic_array::GenericArray};
use anyhow::{Context, Result, anyhow};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

mod decryption;

pub struct Decryptor {
    out_dir: PathBuf,
    decrypted_db: Vec<u8>,
    file_map: HashMap<String, PathBuf>,
}

trait Cipher {
    fn decrypt(&self, buf: &mut [u8], cur_pos: usize, dec_size: usize);
}

impl Decryptor {
    const SFX_ARR: [u8; 8] = [0x5C, 0xBD, 0x98, 0x7C, 0x1C, 0x38, 0x17, 0x8E];
    const CHAR_ARR: [u8; 4] = [0x18, 0x46, 0x30, 0x4D];

    pub fn new(in_dir: PathBuf, out_dir: PathBuf) -> Self {
        if !out_dir.exists() {
            std::fs::create_dir_all(&out_dir)
                .with_context(|| format!("Fail to create output directory: {}", out_dir.display()))
                .unwrap();
        }
        Self {
            file_map: get_name_path_map(Path::new(&in_dir)).unwrap(),
            out_dir,
            decrypted_db: Self::decrypt_db().unwrap(),
        }
    }

    fn gen_db_key(first_half: &[u8], second_half: &[u8]) -> Result<[u8; 16]> {
        let db_key = md5::compute([first_half, second_half, &Self::SFX_ARR].concat());
        format!(
            "{:08X}{:02X}{:02X}",
            u32::from_le_bytes((&db_key[0..4]).try_into().unwrap()),
            u16::from_le_bytes((&db_key[4..6]).try_into().unwrap()),
            u16::from_le_bytes((&db_key[6..8]).try_into().unwrap())
        )
        .as_bytes()
        .try_into()
        .with_context(|| "Key length error")
    }

    fn decrypt_db() -> Result<Vec<u8>> {
        use crate::utils::{get_disk_info, get_mac_addresses};

        let app_dir =
            PathBuf::from(&std::env::var("APPDATA").unwrap_or_default()).join("Tencent\\QQMusic");
        let dat_file_path = get_child_path_by_prfx_and_sfx(&app_dir, "Driver", "dat")?;
        let dat_crc_file_path = get_child_path_by_prfx_and_sfx(&app_dir, "Driver", "dat.crc")?;
        let db_file_path = get_child_path_by_prfx_and_sfx(&app_dir, "Driver", "db")?;
        let db_crc_file_path = get_child_path_by_prfx_and_sfx(&app_dir, "Driver", "db.crc")?;
        let (mut db_file, mut crc_file) = match (
            dat_file_path.len(),
            dat_crc_file_path.len(),
            db_file_path.len(),
            db_crc_file_path.len(),
        ) {
            (0, 0, 0, 0) => Err(anyhow!("Find no database.")),
            (1, 1, 0, 0) => Ok((
                File::open(dat_file_path[0].clone())?,
                File::open(dat_crc_file_path[0].clone())?,
            )),
            (0, 0, 1, 1) => Ok((
                File::open(db_file_path[0].clone())?,
                File::open(db_crc_file_path[0].clone())?,
            )),
            (1, 1, 1, 1) => Err(anyhow!("Find multiple database.")),
            _ => Err(anyhow!("Unexpected database status.")),
        }?;

        let size = db_file
            .read_u32::<LittleEndian>()
            .with_context(|| "Fail to read database size.")? as usize;
        let mut db_cntnt = vec![
            0u8;
            if 0 == size % 16 {
                size
            } else {
                ((size >> 4) + 1) << 4
            }
        ];
        let mut iv = [0u8; 16];

        db_file.read_exact(&mut db_cntnt).with_context(|| {
            format!(
                "Fail to read database content of length {}.",
                db_cntnt.len()
            )
        })?;
        crc_file
            .seek(SeekFrom::Current(12))
            .with_context(|| "Fail to seek in crc file.")?;
        crc_file
            .read_exact(&mut iv)
            .with_context(|| "Fail to read `iv` in crc file.")?;

        let mut blocks: Vec<_> = db_cntnt
            .chunks_exact(16)
            .map(GenericArray::clone_from_slice)
            .collect();
        let first_halves = get_mac_addresses();
        let second_half = get_disk_info();
        for first_half in first_halves {
            let mut test_block = blocks[0];
            let db_key = Self::gen_db_key(&first_half, &second_half).unwrap();
            let mut cipher = cfb_mode::Decryptor::<Aes128>::new(&db_key.into(), &iv.into());

            cipher.decrypt_block_mut(&mut test_block);
            if test_block[4..8] == Self::CHAR_ARR {
                println!("Found database key: {}", String::from_utf8_lossy(&db_key));
                blocks[0] = test_block;
                cipher.decrypt_blocks_mut(&mut blocks[1..]);
                break;
            }
        }
        Ok(blocks.into_iter().flatten().collect::<Vec<u8>>()[4..size].to_vec())
    }

    pub fn decrypt_all(&mut self) -> std::io::Result<()> {
        use crate::decryptor::decryption::Decryption;
        use crate::param::{BLOCK_SIZE, TAIL_SIZE};
        use crate::read_write_buf::ReadWriteBuf;
        use crate::utils::parse_next_kv;

        let mut buf = &self.decrypted_db[..];
        while let Some((next_buf, name, key)) = parse_next_kv(buf) {
            buf = next_buf;
            if let Some(path) = self.file_map.get(String::from_utf8_lossy(name).as_ref()) {
                let file_name = path.file_name().unwrap();
                println!("Decrypting file: {}", file_name.to_string_lossy());
                let mut read_write_buf = ReadWriteBuf::new(
                    path,
                    {
                        let mut path = self.out_dir.join(file_name);
                        path.set_extension("flac");
                        path
                    },
                    BLOCK_SIZE,
                )?;
                let decryption = Decryption::new(key);
                while read_write_buf.get_position() + BLOCK_SIZE + TAIL_SIZE
                    <= read_write_buf.get_file_size()
                {
                    read_write_buf.process_with(BLOCK_SIZE, |data, cur_pos, dec_size| {
                        decryption.decrypt(data, cur_pos, dec_size);
                    })?;
                }
                while read_write_buf.get_position() + TAIL_SIZE < read_write_buf.get_file_size() {
                    read_write_buf.process_with(
                        read_write_buf.get_file_size() - TAIL_SIZE - read_write_buf.get_position(),
                        |data, cur_pos, dec_size| decryption.decrypt(data, cur_pos, dec_size),
                    )?;
                }
                println!("Done!");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    #[test]
    fn test_decrypt_db() {
        File::create("dump.bin")
            .expect("Fail to create file.")
            .write_all(&Decryptor::decrypt_db().expect("Fail to decrypt database."))
            .expect("Fail to write to file.");
    }
}
