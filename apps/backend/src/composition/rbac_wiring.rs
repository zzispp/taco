use std::sync::Arc;

use configuration::Settings;
use rbac::{
    application::{RbacAdminUseCase, RbacAuditedAdminUseCase, RbacCacheRefreshUseCase, RbacService, RbacUseCase},
    infra::{RedisRbacCache, StorageRbacRepository},
};
use storage::Database;

use crate::BackendResult;

pub(super) struct RbacServices {
    pub use_case: Arc<dyn RbacUseCase>,
    pub admin: Arc<dyn RbacAdminUseCase>,
    pub audited_admin: Arc<dyn RbacAuditedAdminUseCase>,
    pub cache_refresher: Arc<dyn RbacCacheRefreshUseCase>,
}

pub(super) async fn build_rbac_services(
    settings: &Settings,
    database: Database,
    observer: taco_tracing::InfrastructureObserver,
) -> BackendResult<RbacServices> {
    let repository = StorageRbacRepository::new(database);
    let cache = RedisRbacCache::connect(&settings.redis_url()?, settings.redis.key_prefix.clone(), observer).await?;
    let service = build_rbac_service(repository, cache).await?;

    Ok(RbacServices {
        use_case: service.clone(),
        admin: service.clone(),
        audited_admin: service.clone(),
        cache_refresher: service,
    })
}

async fn build_rbac_service(
    repository: StorageRbacRepository,
    cache: RedisRbacCache,
) -> BackendResult<Arc<RbacService<StorageRbacRepository, RedisRbacCache>>> {
    let service = Arc::new(RbacService::new(repository, cache));
    service.rebuild_cache().await?;
    Ok(service)
}
