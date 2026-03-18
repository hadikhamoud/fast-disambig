pub mod utils;

pub mod camel;
pub mod sina;

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

    let sina_mod = PyModule::new(py, "sina")?;
    sina::python::register(py, &sina_mod)?;
    m.add_submodule(&sina_mod)?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("fast_disambig.sina", &sina_mod)?;

    Ok(())
}
