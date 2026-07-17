use pyo3::prelude::*;

mod camel;
mod sina;

#[pymodule]
fn fast_disambig(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    let camel_mod = PyModule::new(py, "camel")?;
    camel::register(py, &camel_mod)?;
    m.add_submodule(&camel_mod)?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("fast_disambig.camel", &camel_mod)?;

    let sina_mod = PyModule::new(py, "sina")?;
    sina::register(&sina_mod)?;
    m.add_submodule(&sina_mod)?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("fast_disambig.sina", &sina_mod)?;

    Ok(())
}
