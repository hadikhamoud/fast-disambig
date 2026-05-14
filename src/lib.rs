pub mod utils;

pub mod camel;
pub mod sina;
pub use camel::analyzer;
pub use camel::constants;
pub use camel::downloader;
pub use camel::mle;
pub use camel::morphology_db;
pub use camel::stemmer;
pub use camel::supplement_stems;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn fast_disambig(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    let camel_mod = PyModule::new(py, "camel")?;
    camel::python::register(py, &camel_mod)?;
    m.add_submodule(&camel_mod)?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("fast_disambig.camel", &camel_mod)?;

    Ok(())
}
