use super::*;

mod config;

pub use config::{config_by_key, create_config, delete_config, delete_configs, export_configs, get_config, list_configs, refresh_config_cache, replace_config};

#[require_perms("system:post:export")]
pub async fn export_posts(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestQuery(query): RequestQuery<SystemExportQuery>,
) -> ApiResult<Response> {
    let batch_size = state.export_config.export_batch_config().await?.page_size;
    let filter = post_export_filter(query)?.page_filter(CursorPageRequest::default());
    let mut export = SystemXlsxExport::posts(current_locale())?;
    state
        .system
        .export(
            SystemExportRequest {
                kind: SystemExportKind::Posts(filter),
                batch_size,
            },
            &mut export,
        )
        .await?;
    let artifact = export.finish()?;
    crate::api::operation_audit::record_successful_operation(state.operation_audit.as_ref(), audit_context).await?;
    Ok(xlsx_file_attachment("posts.xlsx", artifact))
}

#[require_perms("system:post:list")]
pub async fn list_posts(State(state): State<SystemApiState>, RequestQuery(query): RequestQuery<SystemListQuery>) -> ApiResult<ApiJson<CursorPage<Post>>> {
    Ok(ok(state.system.page_posts(post_list_filter(query)?).await?))
}
#[require_perms("system:post:query")]
pub async fn get_post(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<Post>> {
    Ok(ok(state.system.get_post(&id).await?))
}

#[require_perms("system:post:list")]
pub async fn post_options(State(state): State<SystemApiState>) -> ApiResult<ApiJson<Vec<Post>>> {
    Ok(ok(state.system.post_options().await?))
}

#[require_perms("system:post:add")]
pub async fn create_post(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestJson(payload): RequestJson<PostInput>,
) -> ApiResult<ApiJson<Post>> {
    let audit = successful_operation_audit(audit_context)?;
    let post = state.system_audited.create_post_with_audit(payload, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(post))
}

#[require_perms("system:post:edit")]
pub async fn replace_post((State(state), audit_context, Path(id), RequestJson(payload)): AuditedResourceJsonRequest<PostInput>) -> ApiResult<ApiJson<Post>> {
    let audit = successful_operation_audit(audit_context)?;
    let post = state.system_audited.replace_post_with_audit(&id, payload, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(post))
}

#[require_perms("system:post:remove")]
pub async fn delete_post(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    Path(id): Path<String>,
) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.system_audited.delete_post_with_audit(&id, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(()))
}

#[require_perms("system:post:remove")]
pub async fn delete_posts(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestJson(payload): RequestJson<BatchIdsInput>,
) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.system_audited.delete_posts_with_audit(payload.ids, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(()))
}

#[require_perms("system:dict:export")]
pub async fn export_dict_types(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestQuery(query): RequestQuery<SystemExportQuery>,
) -> ApiResult<Response> {
    let batch_size = state.export_config.export_batch_config().await?.page_size;
    let filter = dict_type_export_filter(query)?.page_filter(CursorPageRequest::default());
    let mut export = SystemXlsxExport::dict_types(current_locale())?;
    state
        .system
        .export(
            SystemExportRequest {
                kind: SystemExportKind::DictTypes(filter),
                batch_size,
            },
            &mut export,
        )
        .await?;
    let artifact = export.finish()?;
    crate::api::operation_audit::record_successful_operation(state.operation_audit.as_ref(), audit_context).await?;
    Ok(xlsx_file_attachment("dict_types.xlsx", artifact))
}

#[require_perms("system:dict:list")]
pub async fn list_dict_types(
    State(state): State<SystemApiState>,
    RequestQuery(query): RequestQuery<SystemListQuery>,
) -> ApiResult<ApiJson<CursorPage<DictType>>> {
    Ok(ok(state.system.page_dict_types(dict_type_list_filter(query)?).await?))
}
#[require_perms("system:dict:query")]
pub async fn get_dict_type(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<DictType>> {
    Ok(ok(state.system.get_dict_type(&id).await?))
}

#[require_perms("system:dict:list")]
pub async fn dict_type_options(State(state): State<SystemApiState>) -> ApiResult<ApiJson<Vec<DictType>>> {
    Ok(ok(state.system.dict_type_options().await?))
}

#[require_perms("system:dict:remove")]
pub async fn refresh_dict_cache(State(state): State<SystemApiState>, audit_context: Option<Extension<OperationAuditContext>>) -> ApiResult<ApiJson<()>> {
    state.system.refresh_dict_cache().await?;
    crate::api::operation_audit::record_successful_operation(state.operation_audit.as_ref(), audit_context).await?;
    Ok(ok(()))
}

#[require_perms("system:dict:add")]
pub async fn create_dict_type(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestJson(payload): RequestJson<DictTypeInput>,
) -> ApiResult<ApiJson<DictType>> {
    let audit = successful_operation_audit(audit_context)?;
    let item = state.system_audited.create_dict_type_with_audit(payload, audit.record()).await?;
    audit.mark_persisted();
    state.system.refresh_dict_cache().await?;
    Ok(ok(item))
}

#[require_perms("system:dict:edit")]
pub async fn replace_dict_type(
    (State(state), audit_context, Path(id), RequestJson(payload)): AuditedResourceJsonRequest<DictTypeInput>,
) -> ApiResult<ApiJson<DictType>> {
    let audit = successful_operation_audit(audit_context)?;
    let item = state.system_audited.replace_dict_type_with_audit(&id, payload, audit.record()).await?;
    audit.mark_persisted();
    state.system.refresh_dict_cache().await?;
    Ok(ok(item))
}

#[require_perms("system:dict:remove")]
pub async fn delete_dict_type(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    Path(id): Path<String>,
) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.system_audited.delete_dict_type_with_audit(&id, audit.record()).await?;
    audit.mark_persisted();
    state.system.refresh_dict_cache().await?;
    Ok(ok(()))
}

#[require_perms("system:dict:remove")]
pub async fn delete_dict_types(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestJson(payload): RequestJson<BatchIdsInput>,
) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.system_audited.delete_dict_types_with_audit(payload.ids, audit.record()).await?;
    audit.mark_persisted();
    state.system.refresh_dict_cache().await?;
    Ok(ok(()))
}

#[require_perms("system:dict:export")]
pub async fn export_dict_data(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestQuery(query): RequestQuery<SystemExportQuery>,
) -> ApiResult<Response> {
    let batch_size = state.export_config.export_batch_config().await?.page_size;
    let filter = dict_data_export_filter(query)?.page_filter(CursorPageRequest::default());
    let mut export = SystemXlsxExport::dict_data(current_locale())?;
    state
        .system
        .export(
            SystemExportRequest {
                kind: SystemExportKind::DictData(filter),
                batch_size,
            },
            &mut export,
        )
        .await?;
    let artifact = export.finish()?;
    crate::api::operation_audit::record_successful_operation(state.operation_audit.as_ref(), audit_context).await?;
    Ok(xlsx_file_attachment("dict_data.xlsx", artifact))
}

#[require_perms("system:dict:list")]
pub async fn list_dict_data(
    State(state): State<SystemApiState>,
    RequestQuery(query): RequestQuery<SystemListQuery>,
) -> ApiResult<ApiJson<CursorPage<DictData>>> {
    Ok(ok(state.system.page_dict_data(dict_data_list_filter(query)?).await?))
}
#[require_perms("system:dict:query")]
pub async fn get_dict_data(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<DictData>> {
    Ok(ok(state.system.get_dict_data(&id).await?))
}

#[require_perms("system:dict:list")]
pub async fn dict_data_by_type(State(state): State<SystemApiState>, Path(dict_type): Path<String>) -> ApiResult<ApiJson<Vec<DictData>>> {
    Ok(ok(state.system.dict_data_by_type(&dict_type).await?))
}

#[require_perms("system:dict:add")]
pub async fn create_dict_data(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestJson(payload): RequestJson<DictDataInput>,
) -> ApiResult<ApiJson<DictData>> {
    let audit = successful_operation_audit(audit_context)?;
    let item = state.system_audited.create_dict_data_with_audit(payload, audit.record()).await?;
    audit.mark_persisted();
    state.system.refresh_dict_cache().await?;
    Ok(ok(item))
}

#[require_perms("system:dict:edit")]
pub async fn replace_dict_data(
    (State(state), audit_context, Path(id), RequestJson(payload)): AuditedResourceJsonRequest<DictDataInput>,
) -> ApiResult<ApiJson<DictData>> {
    let audit = successful_operation_audit(audit_context)?;
    let item = state.system_audited.replace_dict_data_with_audit(&id, payload, audit.record()).await?;
    audit.mark_persisted();
    state.system.refresh_dict_cache().await?;
    Ok(ok(item))
}

#[require_perms("system:dict:remove")]
pub async fn delete_dict_data(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    Path(id): Path<String>,
) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.system_audited.delete_dict_data_with_audit(&id, audit.record()).await?;
    audit.mark_persisted();
    state.system.refresh_dict_cache().await?;
    Ok(ok(()))
}

#[require_perms("system:dict:remove")]
pub async fn delete_dict_data_batch(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestJson(payload): RequestJson<BatchIdsInput>,
) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.system_audited.delete_dict_data_batch_with_audit(payload.ids, audit.record()).await?;
    audit.mark_persisted();
    state.system.refresh_dict_cache().await?;
    Ok(ok(()))
}
