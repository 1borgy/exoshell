use pyo3::prelude::*;

mod banner;
mod console;
mod mode;
mod shell;

use console::{Action, Console};

#[pymodule]
fn exoshell(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Action>()?;
    m.add_class::<Console>()?;
    Ok(())
}
