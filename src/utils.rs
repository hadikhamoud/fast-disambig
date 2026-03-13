use regex::Regex;
use std::fs::File;
use std::io::BufWriter;
use std::{env, fs, io, path::PathBuf, string::String};

use crate::constants;

pub fn get_or_create_camel_dir() -> Result<PathBuf, io::Error> {
    let curr_home_dir = match env::home_dir() {
        Some(path) => path,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Home directory not found",
            ));
        }
    };
    let camel_dir = curr_home_dir.join(".camel_tools");
    if !camel_dir.exists() {
        fs::create_dir_all(&camel_dir)?;
    }
    Ok(camel_dir)
}

pub fn download_camel_catalogue() -> Result<(), Box<dyn std::error::Error>> {
    let camel_dir = get_or_create_camel_dir()?;
    let catalogue_path = camel_dir.join("catalogue-trial.json");

    let catalogue_file = match File::create_new(&catalogue_path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            eprintln!(
                "Catalogue already exists at {}, skipping download.",
                catalogue_path.display()
            );
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    match ureq::get(constants::CAMEL_DATA_CATALOGUE_URL).call() {
        Ok(mut response) => {
            let writer = BufWriter::new(catalogue_file);

            let response_json: serde_json::Value =
                serde_json::from_str(&response.body_mut().read_to_string()?)?;
            serde_json::to_writer_pretty(writer, &response_json)?;

            Ok(())
        }

        Err(e) => {
            eprintln!("Error getting Catalogue!");
            Err(Box::new(e))
        }
    }
}

pub fn dediac_ar(s: &str) -> String {
    let dediac_re =
        Regex::new(r"[\u{064B}\u{064C}\u{064D}\u{064E}\u{064F}\u{0650}\u{0651}\u{0652}\u{0671}]")
            .unwrap();

    let sub = dediac_re.replace_all(s, "");
    sub.to_string()
}
