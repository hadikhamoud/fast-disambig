use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::string::String;
use zip::ZipArchive;
pub fn dediac_ar(s: &str) -> String {
    let dediac_re =
        Regex::new(r"[\u{064B}\u{064C}\u{064D}\u{064E}\u{064F}\u{0650}\u{0651}\u{0652}\u{0671}]")
            .unwrap();

    let sub = dediac_re.replace_all(s, "");
    sub.to_string()
}

pub fn bytes_to_mib_human_readable(num: usize) -> String {
    let num_mibs = num / (1024 * 1024);
    return num_mibs.to_string() + " MB";
}

pub fn unzip_file(zip_path: &PathBuf, extract_to_path: &PathBuf) -> Result<()> {
    let file = fs::File::open(zip_path)
        .context(format!("Failed to open zip file {}", zip_path.display()))?;
    let mut archive = ZipArchive::new(file).context("Failed to read zip archive")?;

    archive.extract(extract_to_path).context(format!(
        "Failed to extract to {}",
        extract_to_path.display()
    ))?;

    Ok(())
}
