mod map_cache;
mod sync;

pub const REPOSITORY: &str = "aozorabunko_text";

pub use map_cache::{MapCache, MapCacheProgress, update_map, update_map_with_progress};
pub use sync::{GitSyncProgress, sync_repository, sync_repository_simple};

pub struct AnalysedData {
    pub successed: usize,
    pub failed: Vec<String>,
}

impl AnalysedData {
    pub fn new() -> Self {
        Self {
            successed: 0,
            failed: Vec::new(),
        }
    }
    pub fn fail(&mut self, value: String) {
        self.failed.push(value);
    }
    pub fn success(&mut self) {
        self.successed += 1;
    }
    pub fn join(mut self, rhs: Self) -> Self {
        self.successed += rhs.successed;
        self.failed.extend(rhs.failed);
        self
    }
}
