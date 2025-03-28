use super::Decryptor;
use super::Decryption;
use crate::read_write_buf::ReadWriteBuf;
use std::io;
use crate::consts::{SKIP_TO_NAME, NAME_SIZE, SKIP_TO_KEY, KEY_SIZE, BLOCK_SIZE, TAIL_SIZE};

impl Decryptor {
    pub fn decrypt_all(&mut self) -> io::Result<()> {
        let mut pos = 0;
        while pos < self.decrypted_db.len() {
            pos += SKIP_TO_NAME;

            let name = String::from_utf8_lossy(&self.decrypted_db[pos..pos+NAME_SIZE]);
            let filename = match self.name_to_filename.get(name.as_ref()) {
                Some(value) => value,
                None => {
                    pos += NAME_SIZE + SKIP_TO_KEY + KEY_SIZE;
                    continue;
                }
            };

            pos += NAME_SIZE + SKIP_TO_KEY;

            let key = &self.decrypted_db[pos..pos+KEY_SIZE];
            let mut decryption = Decryption::new(key);
            let mut read_write_buf = ReadWriteBuf::new(
                &format!("{}\\{}", self.input_path, filename),
                &format!("{}\\{}", self.output_path, [filename.split(".mflac").next().unwrap(), ".flac"].concat()),
                BLOCK_SIZE)?;
            
            while read_write_buf.get_position() + BLOCK_SIZE + TAIL_SIZE <= read_write_buf.get_file_size() {
                read_write_buf.process_with(BLOCK_SIZE,
                    |data, cur_pos, dec_size| decryption.tweaked_rc4(data, cur_pos, dec_size))?;
            }
            while read_write_buf.get_position() + TAIL_SIZE < read_write_buf.get_file_size() {
                read_write_buf.process_with(read_write_buf.get_file_size() - TAIL_SIZE - read_write_buf.get_position(),
                |data, cur_pos, dec_size| decryption.tweaked_rc4(data, cur_pos, dec_size))?;
            }
            pos += KEY_SIZE;
        }

        Ok(())
    }

}
