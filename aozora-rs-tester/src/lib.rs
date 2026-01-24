mod map_cache;
mod sync;

pub const REPOSITORY: &str = "aozorabunko_text";

pub use map_cache::{MapCache, MapCacheProgress, update_map, update_map_with_progress};
pub use sync::{GitSyncProgress, sync_repository, sync_repository_simple};
