use crate::application::{SystemCache, SystemRepository};

use super::NoSystemCache;

pub struct SystemService<R, C = NoSystemCache> {
    pub(super) repository: R,
    pub(super) cache: C,
}

impl<R: SystemRepository> SystemService<R> {
    pub const fn new(repository: R) -> Self {
        Self {
            repository,
            cache: NoSystemCache,
        }
    }
}

impl<R: SystemRepository, C: SystemCache> SystemService<R, C> {
    pub const fn with_cache(repository: R, cache: C) -> Self {
        Self { repository, cache }
    }
}
