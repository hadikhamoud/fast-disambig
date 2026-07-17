use fast_disambig::sina::{constants, downloader};
use pyo3::prelude::*;

fn python_error(error: impl std::fmt::Display) -> PyErr {
    pyo3::exceptions::PyRuntimeError::new_err(error.to_string())
}

#[pyfunction]
fn asset_url(key: &str) -> Option<String> {
    constants::sina_asset_url(key).map(str::to_owned)
}

#[pyfunction]
fn data_dir() -> PyResult<String> {
    downloader::get_or_create_sina_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(python_error)
}

#[pyfunction]
fn list_datasets() -> PyResult<Vec<String>> {
    downloader::SinaCatalogue::load()
        .map(|catalogue| catalogue.packages.into_keys().collect())
        .map_err(python_error)
}

#[pyfunction]
fn download_dataset(name: &str) -> PyResult<()> {
    downloader::SinaCatalogue::load()
        .and_then(|catalogue| catalogue.download_resource(name))
        .map_err(python_error)
}

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(asset_url, module)?)?;
    module.add_function(wrap_pyfunction!(data_dir, module)?)?;
    module.add_function(wrap_pyfunction!(list_datasets, module)?)?;
    module.add_function(wrap_pyfunction!(download_dataset, module)?)?;
    Ok(())
}
