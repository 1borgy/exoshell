use crate::{Error, Result};
use std::env;
use std::path::PathBuf;

pub fn data_dir() -> Result<PathBuf> {
    let dir = match env::var_os("EXOSHELL_DATA_DIR") {
        Some(path) => PathBuf::from(path),
        None => dirs::data_local_dir()
            .ok_or(Error::Path(
                "could not find data directory, please set EXOSHELL_DATA_DIR manually".into(),
            ))?
            .join("exoshell"),
    };

    if !dir.is_absolute() {
        Err(Error::Path(
            "EXOSHELL_DATA_DIR must be an absolute path".into(),
        ))
    } else {
        Ok(dir)
    }
}

pub fn history_dir() -> Result<PathBuf> {
    let dir = match env::var_os("EXOSHELL_HISTORY_DIR") {
        Some(path) => PathBuf::from(path),
        None => data_dir()?.join("history"),
    };

    if !dir.is_absolute() {
        Err(Error::Path(
            "EXOSHELL_HISTORY_DIR must be an absolute path".into(),
        ))
    } else {
        Ok(dir)
    }
}
