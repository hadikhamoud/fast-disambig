use crate::constants;
use crate::utils;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::PathBuf;
use std::{env, fs};

#[derive(Serialize, Deserialize)]
pub struct CamelCatalogue {
    packages: BTreeMap<String, CamelResource>,
}

#[derive(Serialize, Deserialize)]
pub struct CamelResource {
    name: String,
    description: String,
    private: bool,
    url: Option<String>,
    path: Option<PathBuf>,
    license: Option<String>,
    size: Option<usize>,
    sha256: Option<String>,
    dependencies: Vec<String>,
}

impl CamelCatalogue {
    pub fn download(catalogue_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let camel_dir = get_or_create_camel_dir()?;
        let catalogue_path = camel_dir.join("catalogue.json");
        if !catalogue_path.exists() {
            Self::download(&catalogue_path)?;
        }

        let reader = BufReader::new(File::open(catalogue_path)?);
        let mut catalogue_json: CamelCatalogue = serde_json::from_reader(reader)
            .context("Was not able to read the json into Camel Catalogue")?;
        catalogue_json.packages.retain(|_, res| !res.private);

        Ok(catalogue_json)
    }

    pub fn display(&self) -> Result<(), Box<dyn std::error::Error>> {
        let name_width = self.packages.keys().map(String::len).max().unwrap_or(1);

        let license_width = 10;
        let size_width = 10;
        println!(
            "{:<name_width$}\t{:>size_width$} {:<license_width$}\t{}",
            "Package Name",
            "Size",
            "License",
            "Description",
            name_width = name_width,
            size_width = size_width,
            license_width = license_width,
        );
        println!(
            "{:_<name_width$}\t{:_>size_width$} {:_<license_width$}\t{:_<11}\n",
            "",
            "",
            "",
            "",
            name_width = name_width,
            size_width = size_width,
            license_width = license_width,
        );

        for (k, res) in self.packages.iter() {
            let name = k;
            let size = res.size.unwrap_or(0);
            let mut size_str = "".to_string();
            if size != 0 {
                size_str = utils::bytes_to_mib_human_readable(size);
            }
            println!(
                "{:<name_width$}\t{:>size_width$} {:<license_width$}\t{}",
                name,
                size_str,
                res.license.as_deref().unwrap_or(""),
                res.description,
                name_width = name_width,
                size_width = size_width,
                license_width = license_width,
            );
        }

        Ok(())
    }
}

pub fn get_or_create_camel_dir() -> Result<PathBuf> {
    let curr_home_dir = env::home_dir().context("Home directory not found")?;

    let camel_dir = curr_home_dir.join(".camel_tools");
    if !camel_dir.exists() {
        fs::create_dir_all(&camel_dir).context("Could not create the .camel_tools directory")?;
    }
    Ok(camel_dir)
}
