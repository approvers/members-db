use anyhow::anyhow;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use crate::infra::repository::firestore::{MemberDataRepositoryImpl, OAuth2RepositoryImpl};
use crate::model::MemberListRow;
use crate::service::members::MembersService;

use super::{AppState, HttpError};

pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route("/members", get(get_members))
        .route("/members/:discord_user_id", get(get_member))
}

async fn get_members(
    State(members_service): State<MembersService<MemberDataRepositoryImpl, OAuth2RepositoryImpl>>,
) -> Result<Json<Vec<MemberListRow>>, HttpError> {
    let members = members_service.get_all_members().await?;
    Ok(Json(members))
}

async fn get_member(
    State(members_service): State<MembersService<MemberDataRepositoryImpl, OAuth2RepositoryImpl>>,
    Path(discord_user_id): Path<String>,
) -> Result<Json<MemberListRow>, HttpError> {
    let member = members_service.get_member(&discord_user_id).await?;
    if let Some(member) = member {
        Ok(Json(member))
    } else {
        Err(HttpError(
            StatusCode::NOT_FOUND,
            anyhow!("the member id not found"),
        ))
    }
}
