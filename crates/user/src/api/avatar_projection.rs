use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use futures_util::StreamExt;

use crate::{
    application::{AvatarOwner, AvatarProjection, AvatarProjectionStorage, UserUseCase},
    domain::UserId,
};

const AVATAR_PROJECTION_PATH: &str = "/api/avatars/{user_id}/{version}";
const AVATAR_CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

#[derive(Clone)]
pub struct AvatarProjectionApiState {
    users: Arc<dyn UserUseCase>,
    storage: Arc<dyn AvatarProjectionStorage>,
}

impl AvatarProjectionApiState {
    pub fn new(users: Arc<dyn UserUseCase>, storage: Arc<dyn AvatarProjectionStorage>) -> Self {
        Self { users, storage }
    }
}

pub fn create_avatar_projection_router(state: AvatarProjectionApiState) -> Router {
    Router::new().route(AVATAR_PROJECTION_PATH, get(avatar_projection)).with_state(state)
}

async fn avatar_projection(State(state): State<AvatarProjectionApiState>, Path((user_id, version)): Path<(String, u64)>) -> Result<Response, StatusCode> {
    let user = state.users.get_user(UserId(user_id.clone())).await.map_err(|_| StatusCode::NOT_FOUND)?;
    if user.avatar_version != version {
        return Err(StatusCode::NOT_FOUND);
    }
    let avatar_id = user.avatar_file_id.ok_or(StatusCode::NOT_FOUND)?;
    let content = state
        .storage
        .load_avatar_projection(
            AvatarOwner {
                user_id,
                department_id: user.dept_id,
            },
            avatar_id,
        )
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    projection_response(content)
}

fn projection_response(content: AvatarProjection) -> Result<Response, StatusCode> {
    if !content.content_type.starts_with("image/") {
        return Err(StatusCode::NOT_FOUND);
    }
    let stream = content.body.map(|item| item.map_err(|error| std::io::Error::other(error.to_string())));
    let mut response = Body::from_stream(stream).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, header_value(&content.content_type)?);
    headers.insert(header::CONTENT_LENGTH, header_value(content.content_length)?);
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(AVATAR_CACHE_CONTROL));
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    Ok(response)
}

fn header_value(value: impl ToString) -> Result<HeaderValue, StatusCode> {
    HeaderValue::from_str(&value.to_string()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
