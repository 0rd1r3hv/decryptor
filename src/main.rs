mod decryptor;
mod param;
mod read_write_buf;
mod utils;

fn main() -> anyhow::Result<()> {
    use clap::{Arg, Command};

    let matches = Command::new("Decryptor")
        .version("0.2.0")
        .author("0rd1r3hv")
        .about("Audio decryptor for a specific streaming application")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("INPUT")
                .help("Sets the input directory")
                .required(false),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT")
                .help("Sets the output directory"),
        )
        .get_matches();

    let input = matches
        .get_one::<String>("input")
        .cloned()
        .unwrap_or("input".into());
    let output = matches
        .get_one::<String>("output")
        .cloned()
        .unwrap_or("output".into());

    let mut decryptor = decryptor::Decryptor::new(input.into(), output.into());
    decryptor.decrypt_all()?;
    println!("All done!");
    Ok(())
}
