use crate::application::StoredObject;
use crate::domain::UploadId;

pub(in crate::infra) struct UploadCancellationRequest<'a> {
    pub(in crate::infra) owner_user_id: &'a str,
    pub(in crate::infra) session_id: UploadId,
    pub(in crate::infra) claim_token: &'a str,
}

pub(in crate::infra) struct UploadCompletionAbortRequest<'a> {
    pub(in crate::infra) owner_user_id: &'a str,
    pub(in crate::infra) session_id: UploadId,
    pub(in crate::infra) object: StoredObject,
}

pub(in crate::infra) struct ClaimedUploadCompletionAbortRequest<'a> {
    pub(in crate::infra) session_id: UploadId,
    pub(in crate::infra) claim_token: &'a str,
    pub(in crate::infra) object: StoredObject,
}

impl<'a> UploadCancellationRequest<'a> {
    pub(in crate::infra) const fn new(owner_user_id: &'a str, session_id: UploadId, claim_token: &'a str) -> Self {
        Self {
            owner_user_id,
            session_id,
            claim_token,
        }
    }
}

impl<'a> UploadCompletionAbortRequest<'a> {
    pub(in crate::infra) fn new(owner_user_id: &'a str, session_id: UploadId, object: StoredObject) -> Self {
        Self {
            owner_user_id,
            session_id,
            object,
        }
    }
}

impl<'a> ClaimedUploadCompletionAbortRequest<'a> {
    pub(in crate::infra) fn new(session_id: UploadId, claim_token: &'a str, object: StoredObject) -> Self {
        Self {
            session_id,
            claim_token,
            object,
        }
    }
}
