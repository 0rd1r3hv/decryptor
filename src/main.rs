mod decryptor;
mod param;
mod read_write_buf;
mod utils;

fn main() -> anyhow::Result<()> {
    use clap::{Arg, Command};

    let matches = Command::new("Decryptor")
        .version("0.2.0")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .default_value("input"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .default_value("output"),
        )
        .get_matches();

    let input = matches.get_one::<String>("input").unwrap();
    let output = matches.get_one::<String>("output").unwrap();

    let mut decryptor = decryptor::Decryptor::new(input.into(), output.into());
    decryptor.decrypt_all()?;
    println!("All done!");
    Ok(())
}
