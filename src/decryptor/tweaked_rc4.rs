use super::Decryption;
use std::cmp::min;
use crate::consts::{BLOCK_SIZE, KEY_LEN_MASK, HEADER_SIZE};

impl Decryption {
    fn pseudo_rand(&self, seed: usize) -> usize {
        (self.hash as f64 / (self.key[seed & KEY_LEN_MASK] as usize * (seed + 1)) as f64 * 100.0) as usize & KEY_LEN_MASK
    }

    pub fn tweaked_rc4(&mut self, buf: &mut [u8], cur_pos: usize, dec_size: usize) {
        assert!(dec_size <= BLOCK_SIZE);
        if cur_pos <= HEADER_SIZE {
            for i in 0..min(HEADER_SIZE, dec_size) {
                let pos = self.pseudo_rand(i + cur_pos);
                buf[i] ^= self.key[pos];
            }
            if dec_size > HEADER_SIZE {
                self.dec_part(&mut buf[HEADER_SIZE..], cur_pos + HEADER_SIZE, dec_size - HEADER_SIZE);
            }

        } else {
            let remain = BLOCK_SIZE - (cur_pos % BLOCK_SIZE);
            if dec_size > remain {
                if remain > 0 {
                    self.dec_part(buf, cur_pos, remain);
                }
                self.dec_part(&mut buf[remain..], cur_pos + remain, dec_size - remain);
            } else {
                self.dec_part(buf, cur_pos, dec_size);
            }
        }
    }

    fn dec_part(&mut self, buf: &mut [u8], cur_pos: usize, dec_size: usize) {
        let mut perm = self.sbox.clone();
        let rounds = cur_pos % BLOCK_SIZE + self.pseudo_rand(cur_pos / BLOCK_SIZE);
        let mut i = 0;

        for j in 1..dec_size+rounds+1 {
            i = (i + perm[j & KEY_LEN_MASK] as usize) & KEY_LEN_MASK;
            perm.swap(i, j & KEY_LEN_MASK);
            if j > rounds {
                buf[j - rounds - 1] ^= perm[(perm[i] as usize + perm[j & KEY_LEN_MASK] as usize) & KEY_LEN_MASK];
            }
        }
    }
}