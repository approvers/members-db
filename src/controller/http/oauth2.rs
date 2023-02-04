use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::get;
use axum::Router;
use oauth2::CsrfToken;
use serde::Deserialize;

use crate::infra::repository::firestore::{MemberDataRepositoryImpl, OAuth2RepositoryImpl};
use crate::usecase::members::MembersUseCase;
use crate::usecase::oauth2::OAuth2UseCase;

use super::{AppState, HttpError};

pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route("/discord", get(discord_auth))
        .route("/discord/callback", get(discord_auth_callback))
}

async fn discord_auth(
    State(oauth2_usecase): State<OAuth2UseCase<OAuth2RepositoryImpl>>,
) -> Result<Redirect, HttpError> {
    let auth_url = oauth2_usecase.authenticate().await?;

    Ok(Redirect::to(auth_url.as_str()))
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    code: String,
    state: CsrfToken,
}

async fn discord_auth_callback(
    State(oauth2_usecase): State<OAuth2UseCase<OAuth2RepositoryImpl>>,
    State(members_usecase): State<MembersUseCase<MemberDataRepositoryImpl>>,
    Query(AuthRequest {
        code,
        state: csrf_token,
    }): Query<AuthRequest>,
) -> Result<String, HttpError> {
    let (discord_user_id, access_token, refresh_token) = oauth2_usecase
        .get_token_data(csrf_token.secret().to_owned(), code)
        .await?;
    members_usecase
        .new_member_data(
            discord_user_id,
            access_token.secret().to_owned(),
            refresh_token.secret().to_owned(),
        )
        .await?;
    Ok("Successed to connect your discord account".to_string())
}
