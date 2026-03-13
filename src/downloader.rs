use crate::constants;
use serde_json::Value;
use std::fs::File;
use std::fs::exists;
use std::fs::read_to_string;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::path::PathBuf;
use std::{env, fs, io, string::String};

struct CamelResource {
    name: String,
    url: String,
    path: PathBuf,
    license: String,
    description: String,
    size: f64,
    sha256: String,
    private: bool,
    dependencies: Vec<String>,
}

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

fn download_camel_catalogue() -> Result<(), Box<dyn std::error::Error>> {
    let camel_dir = get_or_create_camel_dir()?;
    let catalogue_path = camel_dir.join("catalogue.json");
    let catalogue_file = File::create(catalogue_path)?;
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

pub fn get_camel_catalogue() -> Result<(), Box<dyn std::error::Error>> {
    let camel_dir = get_or_create_camel_dir()?;
    let catalogue_path = camel_dir.join("catalogue.json");
    if !catalogue_path.exists() {
        download_camel_catalogue()?;
    }

    let reader = BufReader::new(File::open(catalogue_path)?);
    let catalogue_json: Value = serde_json::from_reader(reader)?;
    println!("{}", catalogue_json["components"]);
    Ok(())
}
