use crate::application::{FileAccessScope, FileManagementRepository};
use crate::domain::UploadId;
use crate::{FileError, FileResult};

use super::{FileService, provider_cleanup::AbortUploadOutcome};

impl<R> FileService<R>
where
    R: FileManagementRepository,
{
    pub(super) async fn cancel_managed_upload(&self, actor: FileAccessScope, session_id: UploadId) -> FileResult<()> {
        let (session, _) = self.repository.get_upload_session(&actor, session_id).await?.ok_or(FileError::UploadNotFound)?;
        if session.owner_user_id != actor.user_id && !actor.can_manage_uploads {
            return Err(FileError::Forbidden);
        }
        if session.state != "open" {
            return Err(FileError::UploadNotFound);
        }
        let claim_token = self.repository.claim_upload_cancellation(&session.owner_user_id, session_id).await?;
        self.repository.cancel_upload(&session.owner_user_id, session_id, &claim_token).await?;
        match self.abort_or_enqueue_upload(&session.provider_key, &session.provider_upload_ref).await? {
            AbortUploadOutcome::Aborted => Ok(()),
            AbortUploadOutcome::Queued(error) => Err(error),
        }
    }
}
