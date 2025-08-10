use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::u128;

use crate::error::Result;
use crate::path;

#[derive(Debug)]
pub struct History {
    path: PathBuf,
    entries: Vec<Entry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Entry {
    pub cmd: String,
    count: u64,
    #[serde(alias = "timestamp")]
    ts: u128,
}

impl Entry {
    #[allow(dead_code)] // Unused until Ctrl+R search implemented
    fn recency_factor(&self, now: u128) -> u64 {
        match now.saturating_sub(self.ts) {
            0..3600 => 8,
            3600..86400 => 4,
            86400..604800 => 2,
            604800..=u128::MAX => 1,
        }
    }
}

impl Default for History {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            entries: Vec::new(),
        }
    }
}

impl History {
    pub fn create(name: impl AsRef<str>) -> Result<Self> {
        let path = path::history_dir()?.join(format!("{}.ron", name.as_ref()));
        let self_ = Self {
            path,
            entries: Vec::new(),
        };
        self_.write()?;
        Ok(self_)
    }

    pub fn load_by_name(name: impl AsRef<str>) -> Result<Self> {
        let path = path::history_dir()?.join(format!("{}.ron", name.as_ref()));
        Ok(Self::load(&path)?)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file = fs::File::open(&path)?;
        let contents = io::read_to_string(file)?;
        let entries = ron::from_str(contents.as_str())?;

        log::debug!("loaded history from {:#?}", path);
        let history = Self {
            path: path.into(),
            entries,
        };

        Ok(history)
    }

    pub fn write(&self) -> Result<usize> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&self.path)?;
        let contents = ron::ser::to_string(&self.entries)?;
        let bytes = file.write(contents.as_bytes())?;

        log::debug!("wrote {} bytes to {:?}", bytes, self.path);
        Ok(bytes)
    }

    pub fn update(&mut self, cmd: impl AsRef<str>) -> Result<()> {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        self.add(cmd, now.as_nanos());
        self.sort();
        self.write()?;
        Ok(())
    }

    pub fn add(&mut self, cmd: impl AsRef<str>, ts: u128) {
        match self
            .entries
            .iter_mut()
            .find(|entry| entry.cmd == cmd.as_ref())
        {
            Some(entry) => {
                entry.count += 1;
                entry.ts = ts;
            }
            None => self.entries.push(Entry {
                cmd: cmd.as_ref().into(),
                count: 1,
                ts,
            }),
        }
    }

    pub fn sort(&mut self) {
        self.entries.sort_by_key(|e| e.ts);
    }

    pub fn entries(&mut self) -> &Vec<Entry> {
        &self.entries
    }
}
