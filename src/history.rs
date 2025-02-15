use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::error::Result;
use crate::path;

#[derive(Debug)]
pub struct History {
    path: PathBuf,
    entries: Vec<Entry>,
}

// inspired by zoxide
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Entry {
    pub cmd: String,
    count: u64,
    timestamp: u64,
}

impl Entry {
    fn score(&self, now: u64) -> u64 {
        match now.saturating_sub(self.timestamp) {
            0..3600 => self.count * 4,
            3600..86400 => self.count * 2,
            86400..604800 => self.count / 2,
            604800..=u64::MAX => self.count / 4,
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
    pub fn new(name: impl AsRef<str>) -> Result<Self> {
        let path = path::history_dir()?.join(format!("{}.ron", name.as_ref()));
        Ok(Self {
            path,
            entries: Vec::new(),
        })
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

        log::info!("loaded history from {:#?}", path);
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

        log::info!("wrote {} bytes to {:?}", bytes, self.path);
        Ok(bytes)
    }

    pub fn update(&mut self, cmd: impl AsRef<str>) -> Result<()> {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        self.add(cmd, timestamp.as_secs());
        self.sort()?;
        self.write()?;
        Ok(())
    }

    pub fn add(&mut self, cmd: impl AsRef<str>, timestamp: u64) {
        match self
            .entries
            .iter_mut()
            .find(|entry| entry.cmd == cmd.as_ref())
        {
            Some(entry) => {
                entry.count += 1;
                entry.timestamp = timestamp;
            }
            None => self.entries.push(Entry {
                cmd: cmd.as_ref().into(),
                count: 1,
                timestamp,
            }),
        }
    }

    pub fn sort(&mut self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();

        self.entries.sort_by_key(|e| {
            let score = e.score(now);
            log::info!("cmd: {}, score: {}", e.cmd, score);
            score
        });

        Ok(())
    }

    pub fn entries(&mut self) -> &Vec<Entry> {
        &self.entries
    }
}
