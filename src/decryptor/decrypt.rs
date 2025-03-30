use super::Decryption;
use super::Decryptor;
use crate::consts::{BLOCK_SIZE, TAIL_SIZE};
use crate::read_write_buf::ReadWriteBuf;
use crate::utils::parse_next_kv;
use std::io;

impl Decryptor {
    pub fn decrypt_all(&mut self) -> io::Result<()> {
        let mut buf = &self.decrypted_db[..];
        while let Some((next_buf, name, key)) = parse_next_kv(buf) {
            buf = next_buf;
            let name = String::from_utf8_lossy(name);
            if let Some(filename) = self.name_to_filename.get(name.as_ref()) {
                let decryption = Decryption::new(key);
                let mut read_write_buf = ReadWriteBuf::new(
                    &format!("{}\\{}", self.input_path, filename),
                    &format!(
                        "{}\\{}",
                        self.output_path,
                        [filename.split(".mflac").next().unwrap(), ".flac"].concat()
                    ),
                    BLOCK_SIZE,
                )?;

                while read_write_buf.get_position() + BLOCK_SIZE + TAIL_SIZE
                    <= read_write_buf.get_file_size()
                {
                    read_write_buf.process_with(BLOCK_SIZE, |data, cur_pos, dec_size| {
                        decryption.decrypt(data, cur_pos, dec_size)
                    })?;
                }
                while read_write_buf.get_position() + TAIL_SIZE < read_write_buf.get_file_size() {
                    read_write_buf.process_with(
                        read_write_buf.get_file_size() - TAIL_SIZE - read_write_buf.get_position(),
                        |data, cur_pos, dec_size| decryption.decrypt(data, cur_pos, dec_size),
                    )?;
                }
            }
        }

        Ok(())
    }

}
