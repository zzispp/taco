use async_trait::async_trait;

use crate::application::{RbacCache, RbacCacheRefreshUseCase, RbacRepository, RbacResult};

use super::RbacService;

#[async_trait]
impl<R, C> RbacCacheRefreshUseCase for RbacService<R, C>
where
    R: RbacRepository,
    C: RbacCache,
{
    async fn refresh_after_audited_write(&self) -> RbacResult<()> {
        self.rebuild_cache().await
    }
}
