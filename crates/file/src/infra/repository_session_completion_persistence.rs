use sqlx::{Postgres, QueryBuilder, query, query_as};

use crate::application::{ProviderCleanupKind, ProviderCleanupRecordRequest, StoredObject};
use crate::{FileError, FileResult};

use super::super::repository_provider_cleanup::{cancel_object_cleanup_tx, record_tx};
use super::super::repository_session_support::{map_insert, parent_value};
use super::super::repository_support::{same_physical_object, storage_error};
use super::CompletionContext;

struct CanonicalObject {
    id: String,
    provider_key: String,
    object_key: String,
}

pub(super) async fn adopt_or_insert_object(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    context: &CompletionContext<'_>,
) -> FileResult<(String, Option<StoredObject>)> {
    cancel_object_cleanup_tx(transaction, &context.object.provider_key, &context.object.key).await?;
    if let Some(canonical) = canonical_object(transaction, context.digest, context.size).await? {
        return adopt_canonical_object(transaction, canonical, context).await;
    }
    if insert_object(transaction, context).await? {
        return Ok((context.object.id.to_string(), None));
    }
    let canonical = canonical_object(transaction, context.digest, context.size)
        .await?
        .ok_or_else(|| FileError::Infrastructure("content deduplication object disappeared during completion".into()))?;
    adopt_canonical_object(transaction, canonical, context).await
}

async fn canonical_object(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    digest: crate::domain::ContentDigest,
    size: i64,
) -> FileResult<Option<CanonicalObject>> {
    query_as("SELECT object_id,provider_key,object_key FROM file_object WHERE sha256=$1 AND size_bytes=$2 AND status='active' FOR UPDATE")
        .bind(digest.to_hex())
        .bind(size)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)
        .map(|row: Option<(String, String, String)>| row.map(|(id, provider_key, object_key)| CanonicalObject { id, provider_key, object_key }))
}

async fn insert_object(transaction: &mut sqlx::Transaction<'_, Postgres>, context: &CompletionContext<'_>) -> FileResult<bool> {
    query("INSERT INTO file_object(object_id,provider_key,object_key,size_bytes,sha256,content_type,ref_count,status,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,1,'active',$7,$7) ON CONFLICT DO NOTHING")
        .bind(context.object.id.to_string())
        .bind(context.object.provider_key.as_str())
        .bind(context.object.key.as_str())
        .bind(context.size)
        .bind(context.digest.to_hex())
        .bind(&context.session.content_type)
        .bind(context.now)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)
        .map(|result| result.rows_affected() == 1)
}

async fn adopt_canonical_object(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    canonical: CanonicalObject,
    context: &CompletionContext<'_>,
) -> FileResult<(String, Option<StoredObject>)> {
    query("UPDATE file_object SET ref_count=ref_count+1,updated_at=$2 WHERE object_id=$1")
        .bind(&canonical.id)
        .bind(context.now)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
    let adopted = same_physical_object(&canonical.provider_key, &canonical.object_key, &context.object);
    if !adopted {
        record_tx(
            transaction,
            ProviderCleanupRecordRequest {
                provider_key: &context.object.provider_key,
                kind: ProviderCleanupKind::Object,
                object_key: Some(&context.object.key),
                upload_ref: None,
            },
        )
        .await?;
    }
    Ok((canonical.id, (!adopted).then_some(context.object.clone())))
}

pub(super) async fn insert_file_entry(transaction: &mut sqlx::Transaction<'_, Postgres>, object_id: &str, context: &CompletionContext<'_>) -> FileResult<()> {
    query("INSERT INTO file_entry(entry_id,space_id,parent_id,kind,name,normalized_name,object_id,status,created_by,created_at,updated_by,updated_at) VALUES($1,$2,$3,'file',$4,$5,$6,'active',$7,$8,$7,$8)")
        .bind(context.entry_id.to_string())
        .bind(context.session.space_id.as_str())
        .bind(parent_value(context.session.parent_id))
        .bind(context.session.name.as_str())
        .bind(context.session.name.normalized())
        .bind(object_id)
        .bind(&context.actor.user_id)
        .bind(context.now)
        .execute(&mut **transaction)
        .await
        .map_err(map_insert)?;
    Ok(())
}

pub(super) async fn mark_session_completed(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    context: &CompletionContext<'_>,
    entry_id: crate::domain::FileId,
) -> FileResult<()> {
    let mut update = QueryBuilder::<Postgres>::new(
        "UPDATE file_upload_session SET state='completed',reserved_bytes=0,cleanup_claim_token=NULL,cleanup_claimed_at=NULL,result_entry_id=",
    );
    update
        .push_bind(entry_id.to_string())
        .push(",completed_at=")
        .push_bind(context.now)
        .push(",last_activity_at=")
        .push_bind(context.now)
        .push(" WHERE session_id=")
        .push_bind(context.session_id.to_string())
        .push(" AND state='completing' AND ");
    if let Some(token) = context.claim_token {
        update.push("cleanup_claim_token=").push_bind(token);
    } else {
        update.push("cleanup_claim_token IS NULL");
    }
    let result = update.build().execute(&mut **transaction).await.map_err(storage_error)?;
    if result.rows_affected() == 0 {
        return Err(FileError::UploadNotFound);
    }
    Ok(())
}
