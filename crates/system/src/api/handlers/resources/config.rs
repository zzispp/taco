use super::super::*;

#[require_perms("system:config:export")]
pub async fn export_configs(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestQuery(query): RequestQuery<SystemExportQuery>,
) -> ApiResult<Response> {
    let batch_size = state.export_config.export_batch_config().await?.page_size;
    let filter = config_export_filter(query)?.page_filter(CursorPageRequest::default());
    let mut export = SystemXlsxExport::configs(current_locale())?;
    state
        .system
        .export(
            SystemExportRequest {
                kind: SystemExportKind::Configs(filter),
                batch_size,
            },
            &mut export,
        )
        .await?;
    let artifact = export.finish()?;
    crate::api::operation_audit::record_successful_operation(state.operation_audit.as_ref(), audit_context).await?;
    Ok(xlsx_file_attachment("configs.xlsx", artifact))
}

#[require_perms("system:config:list")]
pub async fn list_configs(
    State(state): State<SystemApiState>,
    RequestQuery(query): RequestQuery<SystemListQuery>,
) -> ApiResult<ApiJson<CursorPage<ConfigItem>>> {
    Ok(ok(state.system.page_configs(config_list_filter(query)?).await?))
}

#[require_perms("system:config:query")]
pub async fn get_config(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<ConfigItem>> {
    Ok(ok(state.system.get_config(&id).await?))
}

#[require_perms("system:config:query")]
pub async fn config_by_key(State(state): State<SystemApiState>, Path(key): Path<String>) -> ApiResult<ApiJson<String>> {
    Ok(ok(state.system.config_by_key(&key).await?))
}

#[require_perms("system:config:remove")]
pub async fn refresh_config_cache(State(state): State<SystemApiState>, audit_context: Option<Extension<OperationAuditContext>>) -> ApiResult<ApiJson<()>> {
    state.system.refresh_config_cache().await?;
    crate::api::operation_audit::record_successful_operation(state.operation_audit.as_ref(), audit_context).await?;
    Ok(ok(()))
}

#[require_perms("system:config:add")]
pub async fn create_config(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestJson(payload): RequestJson<ConfigInput>,
) -> ApiResult<ApiJson<ConfigItem>> {
    let audit = successful_operation_audit(audit_context)?;
    let item = state.system_audited.create_config_with_audit(payload, audit.record()).await?;
    audit.mark_persisted();
    state.system.refresh_config_cache().await?;
    Ok(ok(item))
}

#[require_perms("system:config:edit")]
pub async fn replace_config(
    (State(state), audit_context, Path(id), RequestJson(payload)): AuditedResourceJsonRequest<ConfigInput>,
) -> ApiResult<ApiJson<ConfigItem>> {
    let audit = successful_operation_audit(audit_context)?;
    let item = state.system_audited.replace_config_with_audit(&id, payload, audit.record()).await?;
    audit.mark_persisted();
    state.system.refresh_config_cache().await?;
    Ok(ok(item))
}

#[require_perms("system:config:remove")]
pub async fn delete_config(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    Path(id): Path<String>,
) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.system_audited.delete_config_with_audit(&id, audit.record()).await?;
    audit.mark_persisted();
    state.system.refresh_config_cache().await?;
    Ok(ok(()))
}

#[require_perms("system:config:remove")]
pub async fn delete_configs(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestJson(payload): RequestJson<BatchIdsInput>,
) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.system_audited.delete_configs_with_audit(payload.ids, audit.record()).await?;
    audit.mark_persisted();
    state.system.refresh_config_cache().await?;
    Ok(ok(()))
}
