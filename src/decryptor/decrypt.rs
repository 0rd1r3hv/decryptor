use super::Decryptor;
use std::io;

const BLOCK_SIZE: usize = 0x1400;

impl Decryptor {
    pub fn decrypt(&mut self) -> io::Result<()> {
        while let Ok(size) = self.read() {
            if size == 0 {
                break;
            }
            self.tweaked_rc4(size);
            self.write(size)?;
        }
        Ok(())
    }

    fn calc(&self, seed: usize) -> usize {
        (self.decrypt_key.hash as f64 / (self.decrypt_key.key[seed % self.decrypt_key.key_len] as usize * (seed + 1)) as f64 * 100.0) as usize % self.decrypt_key.key_len
    }

    fn tweaked_rc4(&mut self, dec_size: usize) {
        let cur_pos = self.get_current_position().unwrap();
        if cur_pos <= 0x80 {
            for i in 0..0x80 {
                let pos = self.calc(i + cur_pos as usize);
                self.block[i] ^= self.decrypt_key.key[pos];
            }
            self.dec_part(cur_pos + 0x80, dec_size - 0x80, 0x80);
        } else {
            let remain = BLOCK_SIZE - cur_pos as usize % BLOCK_SIZE;
            if dec_size > remain {
                self.dec_part(cur_pos, remain, 0);
                self.dec_part(cur_pos + remain as u64, dec_size - remain, remain);
            } else {
                self.dec_part(cur_pos, dec_size, 0);
            }
        }
    }

    fn dec_part(&mut self, cur_pos: u64, dec_size: usize, seg_pos: usize) {
        let key_len = self.decrypt_key.key_len;
        let mut perm = self.decrypt_key.sbox.clone();
        let rounds = cur_pos as usize % BLOCK_SIZE + self.calc(cur_pos as usize / BLOCK_SIZE);
        let mut i = 0;

        for j in 1..dec_size+rounds+1 {
            i = (i + perm[j % key_len] as usize) % key_len;
            perm.swap(i, j % key_len);
            if j > rounds {
                self.block[seg_pos + j - rounds - 1] ^= perm[(perm[i] as usize + perm[j % key_len] as usize) % key_len];
            }
        }
    }

}
