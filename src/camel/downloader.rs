use crate::camel::constants;
use crate::utils;
use anyhow::Context;
use anyhow::Result;
use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::{env, fs};

fn lock_data_dir(data_dir: &Path) -> Result<File> {
    let lock = File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(data_dir.join(".fast-disambig.lock"))?;
    lock.lock_exclusive()?;
    Ok(lock)
}

fn resource_path(data_dir: &Path, destination: &str) -> Result<PathBuf> {
    let destination = Path::new(destination);
    let mut resolved = data_dir.to_path_buf();
    let mut has_component = false;
    for component in destination.components() {
        let Component::Normal(component) = component else {
            anyhow::bail!("Invalid resource destination '{}'", destination.display());
        };
        has_component = true;
        resolved.push(component);
        if resolved
            .symlink_metadata()
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            anyhow::bail!(
                "Resource destination contains a symbolic link: '{}'",
                destination.display()
            );
        }
    }
    anyhow::ensure!(
        has_component,
        "Invalid resource destination '{}'",
        destination.display()
    );
    Ok(resolved)
}

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

        let camel_dir = get_or_create_camel_dir()?;
        let _lock = lock_data_dir(&camel_dir)?;
        if self.exists()? {
            return Ok(());
        }

        let dest = resource_path(
            &camel_dir,
            self.destination.as_deref().unwrap_or(&self.name),
        )?;
        let mut tmp_archive = tempfile::NamedTempFile::new_in(&camel_dir)
            .context("Failed to create temporary download file")?;

        let mut response = ureq::get(url)
            .call()
            .context(format!("Failed to download '{}'", self.name))?;

        let mut writer = BufWriter::new(tmp_archive.as_file_mut());

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
        drop(writer);

        if let Some(expected_hash) = &self.sha256 {
            let downloaded_hash = utils::hash(&fs::read(tmp_archive.path())?);
            anyhow::ensure!(
                downloaded_hash == *expected_hash,
                "Checksum mismatch while downloading '{}'",
                self.name
            );
        }

        let parent = dest
            .parent()
            .context("Resource destination has no parent")?;
        fs::create_dir_all(parent)?;
        let staging = tempfile::Builder::new()
            .prefix(".fast-disambig-")
            .tempdir_in(parent)?;
        utils::unzip_file(
            &tmp_archive.path().to_path_buf(),
            &staging.path().to_path_buf(),
        )?;
        if dest.exists() {
            fs::remove_dir_all(&dest)?;
        }
        fs::rename(staging.keep(), &dest)
            .with_context(|| format!("Failed to install resource at {}", dest.display()))?;

        Ok(())
    }
    pub fn exists(&self) -> Result<bool> {
        let camel_dir = get_or_create_camel_dir()?;
        let path = resource_path(
            &camel_dir,
            self.destination
                .as_ref()
                .context("Resource has no path field")?,
        )?;

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
    let camel_dir = if let Some(path) = env::var_os("FAST_DISAMBIG_DATA_DIR") {
        PathBuf::from(path)
    } else if let Some(path) = env::var_os("CAMELTOOLS_DATA") {
        PathBuf::from(path)
    } else {
        env::home_dir()
            .context("Home directory not found")?
            .join(".camel_tools/data")
    };
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

        resource_path(&get_or_create_camel_dir()?, destination)
    }

    pub fn get(catalogue_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        match ureq::get(constants::CAMEL_DATA_CATALOGUE_URL).call() {
            Ok(mut response) => {
                let response_json: serde_json::Value =
                    serde_json::from_str(&response.body_mut().read_to_string()?)?;
                let parent = catalogue_path
                    .parent()
                    .context("Catalogue path has no parent")?;
                let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
                serde_json::to_writer_pretty(temporary.as_file_mut(), &response_json)?;
                temporary.as_file_mut().sync_all()?;
                temporary.persist(catalogue_path)?;
                Ok(())
            }

            Err(e) => {
                eprintln!("Error getting Catalogue!");
                Err(Box::new(e))
            }
        }
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_with_download(true)
    }

    pub fn load_with_download(allow_download: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let camel_dir = get_or_create_camel_dir()?;
        let _lock = lock_data_dir(&camel_dir)?;
        let catalogue_path = camel_dir.join("catalogue.json");
        if !catalogue_path.exists() {
            if !allow_download {
                return Err(anyhow::anyhow!("CAMeL data catalogue is not installed").into());
            }
            Self::get(&catalogue_path)?;
        }

        let reader = BufReader::new(File::open(catalogue_path)?);
        let mut catalogue_json: CamelCatalogue = serde_json::from_reader(reader)
            .context("Was not able to read the json into Camel Catalogue")?;
        catalogue_json.packages.retain(|_, res| !res.private);

        for res in catalogue_json.packages.values_mut() {
            if let Some(dest) = &res.destination {
                res.path = Some(resource_path(&camel_dir, dest)?);
            }
        }

        Ok(catalogue_json)
    }

    pub fn component_dataset_path(
        &self,
        component_path: &[&str],
        dataset: Option<&str>,
    ) -> Result<PathBuf> {
        let destination = self.component_dataset_destination(component_path, dataset)?;
        resource_path(&get_or_create_camel_dir()?, destination)
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

#[cfg(test)]
mod tests {
    use super::resource_path;
    use std::path::Path;

    #[test]
    fn resource_destinations_cannot_escape_data_directory() {
        let root = Path::new("/safe/data");
        assert!(resource_path(root, "morphology_db/calima-msa-r13").is_ok());
        assert!(resource_path(root, "").is_err());
        assert!(resource_path(root, "../outside").is_err());
        assert!(resource_path(root, "/outside").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn resource_destinations_reject_intermediate_symlinks() {
        use std::fs;
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("data");
        let outside = temporary.path().join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        symlink(&outside, root.join("linked")).unwrap();

        assert!(resource_path(&root, "linked/resource").is_err());
    }
}
