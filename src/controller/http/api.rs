use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::infra::repository::firestore::{MemberDataRepositoryImpl, OAuth2RepositoryImpl};
use crate::model::MemberListRow;
use crate::service::members::MembersService;

use super::{AppState, HttpError};

pub(crate) fn route() -> Router<AppState> {
    Router::new().route("/members", get(get_members))
}

async fn get_members(
    State(members_service): State<MembersService<MemberDataRepositoryImpl, OAuth2RepositoryImpl>>,
) -> Result<Json<Vec<MemberListRow>>, HttpError> {
    let members = members_service.get_all_members().await?;
    Ok(Json(members))
}
