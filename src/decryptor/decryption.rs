use crate::decryptor::Cipher;
use crate::param::MAGIC;
use crate::param::{BLOCK_SIZE, HEADER_SIZE, KEY_LEN_MASK};
use base64::{Engine as _, engine::general_purpose};
use std::cmp::min;

enum CipherType {
    MapL(MapL),
    TweakedRC4(TweakedRC4),
}

impl Cipher for CipherType {
    fn decrypt(&self, buf: &mut [u8], cur_pos: usize, dec_size: usize) {
        match self {
            CipherType::MapL(cipher) => cipher.decrypt(buf, cur_pos, dec_size),
            CipherType::TweakedRC4(cipher) => cipher.decrypt(buf, cur_pos, dec_size),
        }
    }
}

impl CipherType {
    pub fn key_len(&self) -> usize {
        match self {
            CipherType::MapL(cipher) => cipher.key.len(),
            CipherType::TweakedRC4(cipher) => cipher.key.len(),
        }
    }

    pub fn key(&self) -> &[u8] {
        match self {
            CipherType::MapL(cipher) => &cipher.key,
            CipherType::TweakedRC4(cipher) => &cipher.key,
        }
    }
    pub fn sbox(&self) -> &[u8] {
        match self {
            CipherType::MapL(cipher) => &cipher.key,
            CipherType::TweakedRC4(cipher) => &cipher.sbox,
        }
    }
    pub fn hash(&self) -> u32 {
        match self {
            CipherType::MapL(_) => 0,
            CipherType::TweakedRC4(cipher) => cipher.hash,
        }
    }
}

pub struct Decryption {
    cipher: CipherType,
}

impl Decryption {
    pub fn new(key: &[u8]) -> Self {
        let key = Self::decrypt_key(&general_purpose::STANDARD.decode(key).unwrap());

        Self {
            cipher: if key.len() < 300 {
                CipherType::MapL(MapL::new(key))
            } else {
                CipherType::TweakedRC4(TweakedRC4::new(key))
            },
        }
    }

    pub fn decrypt(&self, buf: &mut [u8], cur_pos: usize, dec_size: usize) {
        self.cipher.decrypt(buf, cur_pos, dec_size);
    }

    fn decrypt_key(input: &[u8]) -> Vec<u8> {
        let mut key = [0u8; 16];

        for i in 0..8 {
            key[i << 1] = MAGIC[i];
            key[(i << 1) + 1] = input[i];
        }

        [&input[0..8], &tc_tea::decrypt(&input[8..], key).unwrap()].concat()
    }
}

pub struct TweakedRC4 {
    key: Vec<u8>,
    hash: u32,
    sbox: Vec<u8>,
}

impl TweakedRC4 {
    pub fn new(key: Vec<u8>) -> Self {
        let mut hash: u32 = 1;
        for b in key.iter() {
            let tmp = hash.wrapping_mul(*b as u32);
            if tmp == 0 || tmp <= hash {
                break;
            }
            hash = tmp;
        }

        let mut sbox: Vec<u8> = (0..key.len()).map(|i| i as u8).collect();
        let mut i = 0;
        for j in 0..key.len() {
            i = (i + key[j] as usize + sbox[j] as usize) % key.len();
            sbox.swap(i, j);
        }

        Self { key, hash, sbox }
    }

    fn pseudo_rand(&self, seed: usize) -> usize {
        (self.hash as f64 / (self.key[seed & KEY_LEN_MASK] as usize * (seed + 1)) as f64 * 100.0)
            as usize
            & KEY_LEN_MASK
    }

    fn dec_part(&self, buf: &mut [u8], cur_pos: usize, dec_size: usize) {
        let mut perm = self.sbox.clone();
        let rounds = cur_pos % BLOCK_SIZE + self.pseudo_rand(cur_pos / BLOCK_SIZE);
        let mut i = 0;

        for j in 1..dec_size + rounds + 1 {
            i = (i + perm[j & KEY_LEN_MASK] as usize) & KEY_LEN_MASK;
            perm.swap(i, j & KEY_LEN_MASK);
            if j > rounds {
                buf[j - rounds - 1] ^=
                    perm[(perm[i] as usize + perm[j & KEY_LEN_MASK] as usize) & KEY_LEN_MASK];
            }
        }
    }
}

impl Cipher for TweakedRC4 {
    fn decrypt(&self, buf: &mut [u8], cur_pos: usize, dec_size: usize) {
        if cur_pos <= HEADER_SIZE {
            for (i, byte) in buf.iter_mut().take(min(HEADER_SIZE, dec_size)).enumerate() {
                let pos = self.pseudo_rand(i + cur_pos);
                *byte ^= self.key[pos];
            }
            if dec_size > HEADER_SIZE {
                self.dec_part(
                    &mut buf[HEADER_SIZE..],
                    cur_pos + HEADER_SIZE,
                    dec_size - HEADER_SIZE,
                );
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
}

#[repr(transparent)]
pub struct MapL {
    key: Vec<u8>,
}

impl MapL {
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }
}

impl Cipher for MapL {
    fn decrypt(&self, buf: &mut [u8], cur_pos: usize, dec_size: usize) {
        for (i, byte) in buf.iter_mut().take(dec_size).enumerate() {
            let mut offset = cur_pos + i;
            if offset > 0x7FFF {
                offset %= 0x7FFF;
            }
            let idx = (offset * offset + 71214) % self.key.len();
            let rot = (idx + 4) % 8;
            let val = self.key[idx];

            *byte ^= (val >> rot) | (val << rot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Decryption;

    #[test]
    fn test_tweasked_rc4() {
        let decrypt_key = Decryption::new("dkR3eWM4b2Ld1cSWGovVomjqZmzlJQlIfbKf5LssgfXkln683OG0OlrotgDjcBt4lR2unxmHnWNX2wGQV/XDuo1NFbBZilXfUM5T/i/JAJ1Le7Y+iNvU1tiZZ9KXOsbHBU5KATwqP3TQA0Ti9gLfa8TS0TiOS+Q2behhWqFCrl9AUqeNMFRd5rbhGcMqQ+Q/8N6Fl32rXMN2Z4dOYYtsp5kSxPt3sKMRgJuXaV/ZqAqbRQNnhcMyPeIqTV439X49Av8261OSUv5YS2YgbR1aPsTX9+qYdYGdjslr57o4/idpUKPvZ0/dIgaFjewG/qo8ClkmL1w1FnDiKqaymFXl7axby2ohu3DkU0PkJbVA82Q6I/MLuiKC6nn/jyPrfGmWi69cLQJmG8aSf4PvLaDXOZ4oXjSmUId6LfWc4Nwb30ysxAjgjl6q8e3QTWq5kO2Ouvb2ksA25QtNpi8jMHHBySsunWh49UCq+QTvKa1VX3qPa75YMAkCCnKPgKPw68TDynqfaOmoZh/5VnXzWvZX7RaOe8X0jRhleyc8tDXjc1KfHrOJx1G49zfE/wC5Vm+OU+R4N4EQ2k1lFMJgSL8DYnprQi+Ut5ua1v+9GSF8zaBxMmWGwM2X2f8fmBD1DjtifG1zFaazeNaYL8BeYIcbf1lzrXbJtS+3i3ArMNRSBXsq8Sa54oqXwyOmztyxK7jr".as_bytes());
        assert_eq!(decrypt_key.cipher.key(), "vDwyc8obg3BP89z9f5U01MyykZBu70N426u0dRChsaX3FxxP214WN9Gf9W8F9G8p81RhqxzYjn7zx32Dzdj9Lza5x3DiHB0u9ZMc9L66qh0W15E4EvE7B06CF13VrTJt8iqRgxp8NU7m722x499vRi6fubQ4UTwqYkM60onhUlX3UVL05S6yBQ21H5z99oFDBf57tg60P7N30Q4cS8M88Llh48UE5xS03tbgyh99qpD0txT0773zCxwx2Yi9jnf09Ww7325NDoFPhtS28nt97XyBXRC67xt6xamq83bCZ2cMmzmsxB6Q47Jl8lnB2I4h6p7Ph2J9a2mQ896m174KQ6y6Bru05f9HX125J1F6DYmWe48mCF5742z469IMAoq702l62CY3S7K67059dw2980IpJgzGEj5u1tJCnvNApRE9uYQ14jtd8gN37Ujiix48l5NNxJdqFsxUjIMH6Dah5MoLr0Gy4FHL759p215hinE6u41j9E154R199Se1NdY6".as_bytes());
        assert_eq!(decrypt_key.cipher.key_len(), 512);
        assert_eq!(
            decrypt_key.cipher.sbox(),
            vec![
                0x66, 0xBB, 0xFA, 0xEE, 0xAE, 0x63, 0xF9, 0x14, 0xCD, 0x98, 0xA3, 0x84, 0xC8, 0x51,
                0x21, 0x64, 0x6B, 0x9A, 0x2E, 0xD4, 0x89, 0xEB, 0x9C, 0x9D, 0x7A, 0x52, 0xDC, 0xD9,
                0x2C, 0x79, 0x6B, 0xF9, 0x8A, 0x50, 0x78, 0xB8, 0x45, 0xCA, 0x8D, 0x31, 0x0F, 0x12,
                0x4A, 0x72, 0x13, 0x60, 0xB9, 0xC7, 0xF5, 0x41, 0x4C, 0x06, 0x2C, 0x9B, 0x05, 0xF5,
                0xFA, 0x20, 0x71, 0xF4, 0x23, 0x00, 0xEC, 0x9B, 0xB7, 0x85, 0x19, 0xC4, 0x4D, 0x36,
                0x92, 0x02, 0xD3, 0xF0, 0xDE, 0x51, 0x43, 0xD4, 0xC4, 0x57, 0xE1, 0xDA, 0x91, 0x60,
                0x00, 0xF3, 0x8E, 0xB2, 0xAD, 0x45, 0xB4, 0xAF, 0xE8, 0x2D, 0x35, 0x2B, 0xEF, 0xE9,
                0x1B, 0x55, 0x20, 0x56, 0x6D, 0x0A, 0xE3, 0x09, 0x7C, 0x05, 0xAD, 0x48, 0xFB, 0x9E,
                0xB5, 0xD7, 0x0A, 0x3F, 0xB2, 0x6F, 0x69, 0x17, 0x21, 0x83, 0x9F, 0x49, 0x5E, 0xF0,
                0x04, 0x5B, 0x30, 0x03, 0xE0, 0x7C, 0x6D, 0x1C, 0x1C, 0x1D, 0xE6, 0x3C, 0xAC, 0x2E,
                0x67, 0x94, 0x5E, 0x8E, 0xB1, 0x9A, 0x11, 0x4F, 0x2F, 0x8B, 0xFF, 0x3A, 0xDD, 0x04,
                0x66, 0x94, 0x65, 0xF7, 0xFD, 0x3F, 0xCC, 0x01, 0x12, 0x86, 0xAB, 0xC6, 0x25, 0xCC,
                0x33, 0x8C, 0xEB, 0x90, 0x07, 0x67, 0x17, 0xE3, 0x1F, 0x47, 0x25, 0x61, 0x6C, 0x7B,
                0x58, 0x44, 0x4D, 0x7E, 0x62, 0x9C, 0xD2, 0x42, 0x4C, 0x08, 0x0F, 0xE9, 0xCA, 0xB8,
                0x78, 0xDF, 0xEE, 0x37, 0xF6, 0x98, 0xBD, 0xA4, 0x8D, 0xBE, 0xDA, 0x1E, 0x15, 0x59,
                0x81, 0xF8, 0x09, 0x2A, 0x52, 0xD0, 0x3B, 0x50, 0xFF, 0x97, 0x30, 0x33, 0xB6, 0xA5,
                0xD8, 0x87, 0xA1, 0x1E, 0x53, 0xB0, 0x3B, 0x16, 0x18, 0x27, 0x8F, 0xBE, 0x5C, 0xC2,
                0xB0, 0x22, 0x5A, 0xF2, 0xA7, 0x37, 0x4A, 0x2B, 0x88, 0xA2, 0x6A, 0xC9, 0x3D, 0x13,
                0xA0, 0x68, 0xAB, 0x74, 0xF6, 0x58, 0x80, 0x73, 0xA7, 0xDE, 0x38, 0xD7, 0x96, 0x54,
                0x87, 0xE1, 0x1A, 0x48, 0x7E, 0x76, 0x40, 0x5F, 0xCB, 0x07, 0xE7, 0xFC, 0x11, 0x5C,
                0x0C, 0xA6, 0x55, 0xE6, 0x1D, 0x38, 0x5D, 0xC8, 0xA3, 0xA6, 0x69, 0xFE, 0x28, 0x59,
                0x03, 0x62, 0x70, 0x02, 0x84, 0xFC, 0xDD, 0x4F, 0x3E, 0x75, 0x64, 0xEC, 0x9F, 0x99,
                0xB7, 0x95, 0x53, 0xBA, 0xC0, 0xFE, 0xAA, 0x19, 0xB3, 0x7B, 0xCF, 0xD5, 0xBB, 0x74,
                0x8F, 0xE2, 0x54, 0x72, 0x14, 0xC3, 0x76, 0x0E, 0x90, 0x73, 0x0D, 0x40, 0xB6, 0x57,
                0xD0, 0x61, 0x39, 0x2D, 0x24, 0x83, 0x8C, 0x4B, 0x24, 0x93, 0xE5, 0xEA, 0xC3, 0xB5,
                0x4E, 0x5B, 0xC9, 0x10, 0x01, 0xCF, 0x49, 0xA4, 0xAF, 0xC6, 0xF1, 0xC5, 0xE8, 0x6F,
                0xF8, 0x95, 0xB3, 0x7F, 0x26, 0xDF, 0xB9, 0x70, 0x8B, 0xA0, 0x23, 0x5F, 0xA8, 0xDB,
                0x08, 0xA9, 0xFB, 0x71, 0x65, 0xD5, 0x75, 0x3D, 0xC0, 0x56, 0x4B, 0x7F, 0xC2, 0xE4,
                0xBC, 0x32, 0x81, 0x22, 0x36, 0xAA, 0xEA, 0xD1, 0x18, 0x31, 0x0D, 0x39, 0x41, 0x3E,
                0xE4, 0xA8, 0x92, 0xC1, 0xBC, 0xAE, 0x5A, 0x34, 0xED, 0x26, 0xDB, 0x68, 0xF7, 0xD9,
                0x2A, 0xBF, 0xEF, 0x42, 0x77, 0xE5, 0x82, 0x16, 0xE7, 0x93, 0xCE, 0x85, 0x7A, 0x6C,
                0xC7, 0xF1, 0x6A, 0x1B, 0x7D, 0x8A, 0x77, 0xC5, 0x46, 0x97, 0x79, 0x34, 0x5D, 0xCB,
                0xFD, 0xD6, 0x32, 0x1F, 0x47, 0x46, 0x29, 0x7D, 0xF4, 0xA5, 0x6E, 0x0B, 0x35, 0xE2,
                0x91, 0xD2, 0x15, 0x28, 0x0B, 0x86, 0x10, 0x0E, 0x2F, 0xDC, 0x88, 0x44, 0x29, 0xD3,
                0x27, 0x3A, 0xCE, 0xD8, 0xD6, 0xCD, 0xB4, 0xD1, 0x43, 0x3C, 0x6E, 0x9E, 0x96, 0xED,
                0x9D, 0xF3, 0xB1, 0x0C, 0xA9, 0xE0, 0x4E, 0x99, 0xA1, 0xBD, 0xBA, 0xA2, 0x89, 0x63,
                0xAC, 0x80, 0xC1, 0x82, 0x1A, 0x06, 0xF2, 0xBF
            ]
        );
        assert_eq!(decrypt_key.cipher.hash(), 0xA9C562F8);
    }
}
