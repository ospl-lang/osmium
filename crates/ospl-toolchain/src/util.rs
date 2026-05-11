use blake3::Hash;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn hash_file(path: impl AsRef<Path>) -> std::io::Result<Hash> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize())
}
