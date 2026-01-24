use aozora_rs_tester::{MapCache, update_map};
use std::io;

pub struct AppContext {
    pub file_paths: Vec<String>,
    pub current_index: usize,
    pub map_cache: Option<MapCache>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            file_paths: Vec::new(),
            current_index: 0,
            map_cache: None,
        }
    }

    pub fn initialize(&mut self) -> io::Result<()> {
        let current_dir = std::env::current_dir()?;
        let map = update_map(&current_dir)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        self.file_paths = map.paths.clone();
        self.map_cache = Some(map);

        Ok(())
    }

    pub fn current_file(&self) -> Option<&str> {
        self.file_paths.get(self.current_index).map(|s| s.as_str())
    }

    pub fn next_file(&mut self) {
        if self.current_index < self.file_paths.len().saturating_sub(1) {
            self.current_index += 1;
        }
    }

    pub fn prev_file(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
    }

    pub fn load_current_file_content(&self) -> io::Result<String> {
        let Some(path) = self.current_file() else {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No file selected"));
        };

        let read_bin = std::fs::read(path)?;
        let (content, _, _) = encoding_rs::SHIFT_JIS.decode(&read_bin);
        Ok(content.into_owned())
    }
}
