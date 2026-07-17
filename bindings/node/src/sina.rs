use fast_disambig::sina::{constants, downloader};
use napi::bindgen_prelude::{AsyncTask, Env, Task};
use napi::{Error, Result, Status};

fn napi_error(error: impl std::fmt::Display) -> Error {
    Error::new(Status::GenericFailure, error.to_string())
}

#[napi(namespace = "sina", js_name = "assetUrl")]
pub fn asset_url(key: String) -> Option<String> {
    constants::sina_asset_url(&key).map(str::to_owned)
}

#[napi(namespace = "sina", js_name = "dataDir")]
pub fn data_dir() -> Result<String> {
    downloader::get_or_create_sina_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(napi_error)
}

#[napi(namespace = "sina", js_name = "listDatasetsSync")]
pub fn list_datasets_sync() -> Result<Vec<String>> {
    downloader::SinaCatalogue::load()
        .map(|catalogue| catalogue.packages.into_keys().collect())
        .map_err(napi_error)
}

#[napi(
    namespace = "sina",
    js_name = "listDatasets",
    ts_return_type = "Promise<Array<string>>"
)]
pub fn list_datasets() -> AsyncTask<ListDatasetsTask> {
    AsyncTask::new(ListDatasetsTask)
}

pub struct ListDatasetsTask;

impl Task for ListDatasetsTask {
    type Output = Vec<String>;
    type JsValue = Vec<String>;

    fn compute(&mut self) -> Result<Self::Output> {
        list_datasets_sync()
    }

    fn resolve(&mut self, _: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

#[napi(namespace = "sina", js_name = "downloadDatasetSync")]
pub fn download_dataset_sync(name: String) -> Result<()> {
    downloader::SinaCatalogue::load()
        .and_then(|catalogue| catalogue.download_resource(&name))
        .map_err(napi_error)
}

#[napi(
    namespace = "sina",
    js_name = "downloadDataset",
    ts_return_type = "Promise<void>"
)]
pub fn download_dataset(name: String) -> AsyncTask<DownloadDatasetTask> {
    AsyncTask::new(DownloadDatasetTask { name })
}

pub struct DownloadDatasetTask {
    name: String,
}

impl Task for DownloadDatasetTask {
    type Output = ();
    type JsValue = ();

    fn compute(&mut self) -> Result<Self::Output> {
        download_dataset_sync(self.name.clone())
    }

    fn resolve(&mut self, _: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}
