mod decryptor;
mod param;
mod read_write_buf;
mod utils;

fn main() -> anyhow::Result<()> {
    let mut decryptor = decryptor::Decryptor::new(
        "C:\\Users\\meand/decryptor\\".into(),
        "../decryptor\\src".into(),
    );
    decryptor.decrypt_all()?;
    Ok(())
}
