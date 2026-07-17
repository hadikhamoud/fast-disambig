use crate::camel::analyzer::ScoredAnalysis;
use crate::camel::downloader;
use crate::camel::mle;
use crate::camel::morphology_db::MorphologyDB;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct CamelResources {
    pub db: MorphologyDB,
    pub model: HashMap<String, ScoredAnalysis>,
}

impl CamelResources {
    pub fn load(name: &str) -> Result<Self> {
        Self::load_with_download(name, true)
    }

    pub fn load_with_download(name: &str, allow_download: bool) -> Result<Self> {
        let db_path = resolve_resource_path_with_download(name, &["MorphologyDB"], allow_download)?;
        let model_path =
            resolve_resource_path_with_download(name, &["DisambigMLE"], allow_download)?;

        Ok(Self {
            db: MorphologyDB::load(db_path)?,
            model: mle::load_mle_model(model_path)?,
        })
    }
}

pub fn load_morphology_db(name: &str) -> Result<MorphologyDB> {
    MorphologyDB::load(resolve_resource_path(name, &["MorphologyDB"])?)
}

pub fn load_morphology_db_with_download(name: &str, allow_download: bool) -> Result<MorphologyDB> {
    MorphologyDB::load(resolve_resource_path_with_download(
        name,
        &["MorphologyDB"],
        allow_download,
    )?)
}

pub fn resolve_resource_path(path_or_name: &str, component_path: &[&str]) -> Result<PathBuf> {
    resolve_resource_path_with_download(path_or_name, component_path, true)
}

pub fn resolve_resource_path_with_download(
    path_or_name: &str,
    component_path: &[&str],
    allow_download: bool,
) -> Result<PathBuf> {
    let direct_path = Path::new(path_or_name);
    if direct_path.exists() {
        return Ok(direct_path.to_path_buf());
    }

    let camel_dir = downloader::get_or_create_camel_dir()?;
    let path = camel_dir.join(path_or_name);
    if path.exists() {
        return Ok(path);
    }

    let catalogue = downloader::CamelCatalogue::load_with_download(allow_download)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let installed_path = catalogue.component_dataset_path(component_path, Some(path_or_name))?;
    if installed_path.exists() {
        return Ok(installed_path);
    }
    anyhow::ensure!(
        allow_download,
        "Dataset '{path_or_name}' is not installed and downloads are disabled"
    );

    catalogue
        .ensure_component_dataset(component_path, Some(path_or_name))
        .with_context(|| format!("Failed to download '{path_or_name}'"))
}
