use crate::constants;
use crate::utils;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::{env, fs};

#[derive(Serialize, Deserialize)]
pub struct CamelCatalogue {
    pub packages: BTreeMap<String, CamelResource>,
    pub components: Value,
}

#[derive(Serialize, Deserialize)]
pub struct CamelResource {
    name: String,
    description: String,
    private: bool,
    destination: Option<String>,
    url: Option<String>,
    pub path: Option<PathBuf>,
    license: Option<String>,
    size: Option<usize>,
    sha256: Option<String>,
    dependencies: Vec<String>,
    files: Option<Vec<CamelResourceFile>>,
}

#[derive(Serialize, Deserialize)]
pub struct CamelResourceFile {
    path: String,
    sha256: String,
    size: usize,
}

impl CamelResource {
    pub fn download(&self) -> Result<()> {
        let url = match self.url.as_ref() {
            Some(u) => u,
            None => return Ok(()),
        };

        if self.exists()? {
            return Ok(());
        }

        let camel_dir = get_or_create_camel_dir()?;
        let dest = camel_dir.join(self.destination.as_deref().unwrap_or(&self.name));
        let tmp_dest =
            PathBuf::from("/tmp").join(self.destination.as_deref().unwrap_or(&self.name));

        if let Some(parent) = tmp_dest.parent() {
            fs::create_dir_all(parent).context(format!(
                "Failed to create temp directory {}",
                parent.display()
            ))?;
        }

        let mut response = ureq::get(url)
            .call()
            .context(format!("Failed to download '{}'", self.name))?;

        let mut writer = BufWriter::new(
            File::create(&tmp_dest)
                .context(format!("Failed to create file at {}", tmp_dest.display()))?,
        );

        let total = self.size.unwrap_or(0);
        let mut downloaded: usize = 0;
        let bar_width: usize = 60;

        println!("Downloading {}...", self.name);

        let mut buf = [0u8; 8192];
        let mut reader = response.body_mut().as_reader();
        loop {
            let n = reader
                .read(&mut buf)
                .context("Failed to read from response stream")?;
            if n == 0 {
                break;
            }
            writer
                .write_all(&buf[..n])
                .context("Failed to write to file")?;

            downloaded += n;
            if total > 0 {
                let pct = (downloaded as f64 / total as f64 * 100.0).min(100.0) as usize;
                let filled = pct * bar_width / 100;
                let empty = bar_width - filled;
                let dl = utils::bytes_to_mib_human_readable(downloaded);
                let tot = utils::bytes_to_mib_human_readable(total);
                print!(
                    "\r{:<30} [{:*<filled$}{: <empty$}] {:>3}% {dl}/{tot}",
                    self.name,
                    "",
                    "",
                    pct,
                    filled = filled,
                    empty = empty,
                );
                io::stdout().flush().ok();
            }
        }
        println!();
        writer.flush().context("Failed to flush file")?;
        fs::create_dir_all(&dest)
            .context(format!("Failed to create destination {}", dest.display()))?;
        utils::unzip_file(&tmp_dest, &dest)?;
        fs::remove_file(&tmp_dest).context("Failed to clean up temp file")?;

        Ok(())
    }
    pub fn exists(&self) -> Result<bool> {
        let camel_dir = get_or_create_camel_dir()?;
        let path = camel_dir.join(
            self.destination
                .as_ref()
                .context("Resource has no path field")?,
        );

        if !path.exists() {
            return Ok(false);
        }

        let expected_files = match self.files.as_ref() {
            Some(files) => files,
            None => return Ok(true),
        };

        for expected in expected_files {
            let file_path = path.join(&expected.path);

            if !file_path.exists() {
                return Ok(false);
            }
            let file_as_bytes = fs::read(&file_path)?;
            let file_hash = utils::hash(&file_as_bytes);
            if file_hash != expected.sha256 {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

pub fn get_or_create_camel_dir() -> Result<PathBuf> {
    let curr_home_dir = env::home_dir().context("Home directory not found")?;

    let camel_dir = curr_home_dir.join(".camel_tools/data");
    if !camel_dir.exists() {
        fs::create_dir_all(&camel_dir).context("Could not create the .camel_tools directory")?;
    }
    Ok(camel_dir)
}

impl CamelCatalogue {
    fn component_entry<'a>(&'a self, component_path: &[&str]) -> Result<&'a Value> {
        let mut current = &self.components;
        for segment in component_path {
            current = current.get(*segment).context(format!(
                "Component '{}' not found in catalogue",
                component_path.join(".")
            ))?;
        }
        Ok(current)
    }

    fn component_dataset_destination<'a>(
        &'a self,
        component_path: &[&str],
        dataset: Option<&str>,
    ) -> Result<&'a str> {
        let component = self.component_entry(component_path)?;
        let datasets = component.get("datasets").context(format!(
            "Component '{}' has no datasets entry",
            component_path.join(".")
        ))?;
        let dataset_name = match dataset {
            Some(name) => name,
            None => component
                .get("default")
                .and_then(Value::as_str)
                .context(format!(
                    "Component '{}' has no default dataset",
                    component_path.join(".")
                ))?,
        };

        datasets
            .get(dataset_name)
            .and_then(|value| value.get("path"))
            .and_then(Value::as_str)
            .context(format!(
                "Dataset '{}' not found for component '{}'",
                dataset_name,
                component_path.join(".")
            ))
    }

    pub fn ensure_component_dataset(
        &self,
        component_path: &[&str],
        dataset: Option<&str>,
    ) -> Result<PathBuf> {
        let destination = self.component_dataset_destination(component_path, dataset)?;
        let package_name = self
            .packages
            .iter()
            .find_map(|(name, resource)| {
                (resource.destination.as_deref() == Some(destination)).then_some(name.as_str())
            })
            .context(format!(
                "No package found for destination '{}' in catalogue",
                destination
            ))?;

        self.download_resource(package_name)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        Ok(get_or_create_camel_dir()?.join(destination))
    }

    pub fn get(catalogue_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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
            Self::get(&catalogue_path)?;
        }

        let reader = BufReader::new(File::open(catalogue_path)?);
        let mut catalogue_json: CamelCatalogue = serde_json::from_reader(reader)
            .context("Was not able to read the json into Camel Catalogue")?;
        catalogue_json.packages.retain(|_, res| !res.private);

        for res in catalogue_json.packages.values_mut() {
            if let Some(dest) = &res.destination {
                res.path = Some(camel_dir.join(dest));
            }
        }

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
    pub fn download_resource(&self, resource_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let resource = self.packages.get(resource_name).context(format!(
            "Package '{}' not found in catalogue",
            resource_name
        ))?;
        for dep in &resource.dependencies {
            self.download_resource(dep)?;
        }
        resource.download()?;
        Ok(())
    }
}
