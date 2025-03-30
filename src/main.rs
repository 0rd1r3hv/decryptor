mod consts;
mod decryptor;
mod read_write_buf;
mod utils;
use decryptor::Decryptor;

fn main() {
    let mut decryptor = Decryptor::new(".", ".").unwrap();
    decryptor.decrypt_all().unwrap();
}
