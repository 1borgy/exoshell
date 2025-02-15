use pyo3::prelude::*;

mod banner;
mod console;
mod error;
mod history;
mod mode;
mod path;
mod shell;

use console::{Action, Console};

pub use error::{Error, Result};

#[pymodule]
fn exoshell(m: &Bound<'_, PyModule>) -> PyResult<()> {
    env_logger::init();
    m.add_class::<Action>()?;
    m.add_class::<Console>()?;
    Ok(())
}
