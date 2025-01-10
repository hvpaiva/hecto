use std::fs;

use crate::error::Result;

#[derive(Debug, Default, Clone)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn load(filename: &str) -> Result<Self> {
        let file_contents = fs::read_to_string(filename)?;
        Ok(Self {
            lines: file_contents.lines().map(str::to_string).collect(),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(String::as_str)
    }
}
