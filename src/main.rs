mod read_write_buf;
mod decryptor;
mod utils;
mod consts;
use decryptor::Decryptor;

fn main() {
    let mut decryptor = Decryptor::new(
        ".",
        ".",)
        .unwrap();
    decryptor.decrypt_all().unwrap();
}
