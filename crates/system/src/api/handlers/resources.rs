use super::*;

#[require_perms("system:post:export")]
pub async fn export_posts(State(state): State<SystemApiState>, RequestQuery(query): RequestQuery<SystemExportQuery>) -> ApiResult<Response> {
    let items = all_export_posts(&state, post_export_filter(query)?).await?;
    Ok(xlsx_attachment("posts.xlsx", export_posts_xlsx(&items, current_locale())?))
}

#[require_perms("system:post:list")]
pub async fn list_posts(State(state): State<SystemApiState>, RequestQuery(query): RequestQuery<SystemListQuery>) -> ApiResult<ApiJson<Page<Post>>> {
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
pub async fn create_post(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<PostInput>) -> ApiResult<ApiJson<Post>> {
    Ok(ok(state.system.create_post(payload).await?))
}

#[require_perms("system:post:edit")]
pub async fn replace_post(
    State(state): State<SystemApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<PostInput>,
) -> ApiResult<ApiJson<Post>> {
    Ok(ok(state.system.replace_post(&id, payload).await?))
}

#[require_perms("system:post:remove")]
pub async fn delete_post(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.system.delete_post(&id).await?;
    Ok(ok(()))
}

#[require_perms("system:post:remove")]
pub async fn delete_posts(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<BatchIdsInput>) -> ApiResult<ApiJson<()>> {
    state.system.delete_posts(payload.ids).await?;
    Ok(ok(()))
}

#[require_perms("system:dict:export")]
pub async fn export_dict_types(State(state): State<SystemApiState>, RequestQuery(query): RequestQuery<SystemExportQuery>) -> ApiResult<Response> {
    let items = all_export_dict_types(&state, dict_type_export_filter(query)?).await?;
    Ok(xlsx_attachment("dict_types.xlsx", export_dict_types_xlsx(&items, current_locale())?))
}

#[require_perms("system:dict:list")]
pub async fn list_dict_types(State(state): State<SystemApiState>, RequestQuery(query): RequestQuery<SystemListQuery>) -> ApiResult<ApiJson<Page<DictType>>> {
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
pub async fn refresh_dict_cache(State(state): State<SystemApiState>) -> ApiResult<ApiJson<()>> {
    state.system.refresh_dict_cache().await?;
    Ok(ok(()))
}

#[require_perms("system:dict:add")]
pub async fn create_dict_type(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<DictTypeInput>) -> ApiResult<ApiJson<DictType>> {
    Ok(ok(state.system.create_dict_type(payload).await?))
}

#[require_perms("system:dict:edit")]
pub async fn replace_dict_type(
    State(state): State<SystemApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<DictTypeInput>,
) -> ApiResult<ApiJson<DictType>> {
    Ok(ok(state.system.replace_dict_type(&id, payload).await?))
}

#[require_perms("system:dict:remove")]
pub async fn delete_dict_type(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.system.delete_dict_type(&id).await?;
    Ok(ok(()))
}

#[require_perms("system:dict:remove")]
pub async fn delete_dict_types(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<BatchIdsInput>) -> ApiResult<ApiJson<()>> {
    state.system.delete_dict_types(payload.ids).await?;
    Ok(ok(()))
}

#[require_perms("system:dict:export")]
pub async fn export_dict_data(State(state): State<SystemApiState>, RequestQuery(query): RequestQuery<SystemExportQuery>) -> ApiResult<Response> {
    let items = all_export_dict_data(&state, dict_data_export_filter(query)?).await?;
    Ok(xlsx_attachment("dict_data.xlsx", export_dict_data_xlsx(&items, current_locale())?))
}

#[require_perms("system:dict:list")]
pub async fn list_dict_data(State(state): State<SystemApiState>, RequestQuery(query): RequestQuery<SystemListQuery>) -> ApiResult<ApiJson<Page<DictData>>> {
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
pub async fn create_dict_data(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<DictDataInput>) -> ApiResult<ApiJson<DictData>> {
    Ok(ok(state.system.create_dict_data(payload).await?))
}

#[require_perms("system:dict:edit")]
pub async fn replace_dict_data(
    State(state): State<SystemApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<DictDataInput>,
) -> ApiResult<ApiJson<DictData>> {
    Ok(ok(state.system.replace_dict_data(&id, payload).await?))
}

#[require_perms("system:dict:remove")]
pub async fn delete_dict_data(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.system.delete_dict_data(&id).await?;
    Ok(ok(()))
}

#[require_perms("system:dict:remove")]
pub async fn delete_dict_data_batch(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<BatchIdsInput>) -> ApiResult<ApiJson<()>> {
    state.system.delete_dict_data_batch(payload.ids).await?;
    Ok(ok(()))
}

#[require_perms("system:config:export")]
pub async fn export_configs(State(state): State<SystemApiState>, RequestQuery(query): RequestQuery<SystemExportQuery>) -> ApiResult<Response> {
    let items = all_export_configs(&state, config_export_filter(query)?).await?;
    Ok(xlsx_attachment("configs.xlsx", export_configs_xlsx(&items, current_locale())?))
}

#[require_perms("system:config:list")]
pub async fn list_configs(State(state): State<SystemApiState>, RequestQuery(query): RequestQuery<SystemListQuery>) -> ApiResult<ApiJson<Page<ConfigItem>>> {
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
pub async fn refresh_config_cache(State(state): State<SystemApiState>) -> ApiResult<ApiJson<()>> {
    state.system.refresh_config_cache().await?;
    Ok(ok(()))
}

#[require_perms("system:config:add")]
pub async fn create_config(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<ConfigInput>) -> ApiResult<ApiJson<ConfigItem>> {
    Ok(ok(state.system.create_config(payload).await?))
}

#[require_perms("system:config:edit")]
pub async fn replace_config(
    State(state): State<SystemApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<ConfigInput>,
) -> ApiResult<ApiJson<ConfigItem>> {
    Ok(ok(state.system.replace_config(&id, payload).await?))
}

#[require_perms("system:config:remove")]
pub async fn delete_config(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.system.delete_config(&id).await?;
    Ok(ok(()))
}

#[require_perms("system:config:remove")]
pub async fn delete_configs(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<BatchIdsInput>) -> ApiResult<ApiJson<()>> {
    state.system.delete_configs(payload.ids).await?;
    Ok(ok(()))
}
