use crate::arena::{ArenaFull, ArenaId};
use moka::future::{Cache, CacheBuilder};
use std::sync::Arc;
use std::time::Duration;

pub struct Repo {
    pub cache: Cache<ArenaId, Arc<ArenaFull>>,
}

impl Repo {
    pub fn new() -> Repo {
        Repo {
            cache: CacheBuilder::new(4096) // lots of ongoing tournaments (usermade)
                .time_to_live(Duration::from_secs(60))
                .build(),
        }
    }

    pub fn get(&self, id: ArenaId) -> Option<Arc<ArenaFull>> {
        self.cache.get(&id)
    }

    pub async fn put(&self, full: ArenaFull) {
        self.cache.insert(full.id.clone(), Arc::new(full)).await
    }
}

impl Default for Repo {
    fn default() -> Self {
        Repo::new()
    }
}
