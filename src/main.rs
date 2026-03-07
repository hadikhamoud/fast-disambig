use std::fs;
use std::io::ErrorKind;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_content = match fs::read_to_string(
        "/Users/hadihamoud/.camel_tools/data/disambig_mle/calima-msa-r13/model.json",
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

    let parsed: serde_json::Value = serde_json::from_str(&file_content)?;
    let parsed = parsed.as_object();

    for (key, value) in parsed.unwrap() {
        println!("{key}");
        println!("{value}");
    }

    Ok(())
}
