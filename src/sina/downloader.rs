use crate::sina::constants::sina_asset_url;
use crate::utils;
use anyhow::Context;
use anyhow::Result;
use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;
use serde_json;
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
pub struct SinaCatalogue {
    pub packages: BTreeMap<String, SinaResource>,
}

#[derive(Serialize, Deserialize)]
pub struct SinaResource {
    name: String,
    description: String,
    private: bool,
    destination: Option<String>,
    url: Option<String>,
    pub path: Option<PathBuf>,
    license: Option<String>,
    size: Option<usize>,
    sha256: String,
}

impl SinaResource {
    pub fn download(&self) -> Result<()> {
        let url = match self.url.as_ref() {
            Some(u) => u,
            None => return Ok(()),
        };

        let sina_dir = get_or_create_sina_dir()?;
        let _lock = lock_data_dir(&sina_dir)?;
        if self.exists()? {
            return Ok(());
        }

        let dest = resource_path(&sina_dir, self.destination.as_deref().unwrap_or(&self.name))?;

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).context(format!(
                "Failed to create temp directory {}",
                parent.display()
            ))?;
        }

        let mut response = ureq::get(url)
            .call()
            .context(format!("Failed to download '{}'", self.name))?;

        let mut temporary = tempfile::NamedTempFile::new_in(
            dest.parent()
                .context("Resource destination has no parent")?,
        )?;
        let mut writer = BufWriter::new(temporary.as_file_mut());

        let total = self.size.unwrap_or(0);
        let mut downloaded: usize = 0;
        let bar_width: usize = 100;

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

        let downloaded_hash = utils::hash(&fs::read(temporary.path())?);
        anyhow::ensure!(
            downloaded_hash == self.sha256,
            "Checksum mismatch while downloading '{}'",
            self.name
        );
        temporary.persist(&dest)?;

        Ok(())
    }
    pub fn exists(&self) -> Result<bool> {
        println!("Checking if {} exists", self.name);
        let sina_dir = get_or_create_sina_dir()?;
        let path = resource_path(
            &sina_dir,
            self.destination
                .as_ref()
                .context("Resource has no path field")?,
        )?;

        if !path.exists() {
            println!("directory does not exist in the first place");
            return Ok(false);
        }

        let file_as_bytes = fs::read(&path)?;
        let file_hash = utils::hash(&file_as_bytes);
        if file_hash != self.sha256 {
            return Ok(false);
        }

        println!("{} already exists, skipping...", self.name);

        Ok(true)
    }
}

pub fn get_or_create_sina_dir() -> Result<PathBuf> {
    let sina_dir = if let Some(path) = env::var_os("FAST_DISAMBIG_SINA_DATA_DIR") {
        PathBuf::from(path)
    } else {
        env::home_dir()
            .context("Home directory not found")?
            .join(".sinatools")
    };
    if !sina_dir.exists() {
        fs::create_dir_all(&sina_dir).context("Could not create the .sinatools directory")?;
    }
    Ok(sina_dir)
}

impl SinaCatalogue {
    pub fn get(catalogue_path: &PathBuf) -> Result<()> {
        let catalogue_url = sina_asset_url("catalogue").context("could not find catalogue URL")?;
        match ureq::get(catalogue_url).call() {
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
                Err(anyhow::Error::new(e))
            }
        }
    }

    pub fn load() -> Result<Self> {
        let sina_dir = get_or_create_sina_dir()?;
        let _lock = lock_data_dir(&sina_dir)?;
        let catalogue_path = sina_dir.join("catalogue.json");
        if !catalogue_path.exists() {
            Self::get(&catalogue_path)?;
        }

        let reader = BufReader::new(File::open(catalogue_path)?);
        let mut catalogue_json: SinaCatalogue = serde_json::from_reader(reader)
            .context("Was not able to read the json into Sina Catalogue")?;
        catalogue_json.packages.retain(|_, res| !res.private);

        for res in catalogue_json.packages.values_mut() {
            if let Some(dest) = &res.destination {
                res.path = Some(resource_path(&sina_dir, dest)?);
            }
        }

        Ok(catalogue_json)
    }

    pub fn display(&self) -> Result<()> {
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
    pub fn download_resource(&self, resource_name: &str) -> Result<()> {
        let resource = self.packages.get(resource_name).context(format!(
            "Package '{}' not found in catalogue",
            resource_name
        ))?;
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
        assert!(resource_path(root, "models/morph.json").is_ok());
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
