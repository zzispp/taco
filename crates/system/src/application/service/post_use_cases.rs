use kernel::pagination::CursorPage;

use crate::{
    application::{PostListFilter, SystemCache, SystemCursorCodec, SystemError, SystemRepository, SystemResult},
    domain::{Post, PostInput},
};

use super::{SystemService, validation::*};

impl<R: SystemRepository, C: SystemCache> SystemService<R, C> {
    pub(super) async fn page_posts_impl(&self, filter: PostListFilter) -> SystemResult<CursorPage<Post>> {
        let filter = sanitize_post_filter(filter);
        validate_page(filter.page.clone())?;
        SystemCursorCodec::post(&filter)?.decode(&filter.page)?;
        self.repository.page_posts(filter).await
    }

    pub(super) async fn get_post_impl(&self, id: &str) -> SystemResult<Post> {
        self.repository.find_post(id).await?.ok_or(SystemError::NotFound)
    }

    pub(super) async fn post_options_impl(&self) -> SystemResult<Vec<Post>> {
        self.repository.post_options().await
    }

    pub(super) async fn create_post_impl(&self, input: PostInput) -> SystemResult<Post> {
        reject_duplicate_post(&self.repository, &input, None).await?;
        self.repository.create_post(input).await
    }

    pub(super) async fn replace_post_impl(&self, id: &str, input: PostInput) -> SystemResult<Post> {
        reject_duplicate_post(&self.repository, &input, Some(id)).await?;
        self.repository.replace_post(id, input).await
    }

    pub(super) async fn delete_post_impl(&self, id: &str) -> SystemResult<()> {
        reject_post_delete(&self.repository, id).await?;
        self.repository.delete_post(id).await
    }

    pub(super) async fn delete_posts_impl(&self, ids: Vec<String>) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        for id in &ids {
            reject_post_delete(&self.repository, id).await?;
        }
        self.repository.delete_posts(&ids).await
    }
}
