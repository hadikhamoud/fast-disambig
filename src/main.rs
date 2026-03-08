use std::fs;
use std::io::ErrorKind;

use fast_disambig::utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = match fs::read_to_string(
        "/Users/hadihamoud/.camel_tools/data/morphology_db/calima-msa-r13/morphology.db",
    ) {
        Ok(content) => content,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                eprintln!("file not found!");
                return Err(error.into());
            }
            _ => {
                eprintln!("Other error: {error}");
                return Err(error.into());
            }
        },
    };
    Ok(())
}
