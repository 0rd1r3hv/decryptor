mod decryptor;
use decryptor::Decryptor;
const DEFAULT_SEGMENT_SIZE: usize = 4096;
fn main() {
    let mut decryptor = Decryptor::new(
        "../input",
        "../output",
        DEFAULT_SEGMENT_SIZE,
        "vDwyc8obg3BP89z9f5U01MyykZBu70N426u0dRChsaX3FxxP214WN9Gf9W8F9G8p81RhqxzYjn7zx32Dzdj9Lza5x3DiHB0u9ZMc9L66qh0W15E4EvE7B06CF13VrTJt8iqRgxp8NU7m722x499vRi6fubQ4UTwqYkM60onhUlX3UVL05S6yBQ21H5z99oFDBf57tg60P7N30Q4cS8M88Llh48UE5xS03tbgyh99qpD0txT0773zCxwx2Yi9jnf09Ww7325NDoFPhtS28nt97XyBXRC67xt6xamq83bCZ2cMmzmsxB6Q47Jl8lnB2I4h6p7Ph2J9a2mQ896m174KQ6y6Bru05f9HX125J1F6DYmWe48mCF5742z469IMAoq702l62CY3S7K67059dw2980IpJgzGEj5u1tJCnvNApRE9uYQ14jtd8gN37Ujiix48l5NNxJdqFsxUjIMH6Dah5MoLr0Gy4FHL759p215hinE6u41j9E154R199Se1NdY6").unwrap();
    decryptor.decrypt().unwrap();
}
