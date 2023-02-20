use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::get;
use axum::Router;
use oauth2::CsrfToken;
use serde::Deserialize;

use crate::infra::repository::firestore::{MemberDataRepositoryImpl, OAuth2RepositoryImpl};
use crate::usecase::oauth2::OAuth2UseCase;

use super::{AppState, HttpError};

#[tracing::instrument]
pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route("/discord", get(discord_auth))
        .route("/discord/callback", get(discord_auth_callback))
}

#[tracing::instrument(skip(oauth2_usecase))]
async fn discord_auth(
    State(oauth2_usecase): State<OAuth2UseCase<MemberDataRepositoryImpl, OAuth2RepositoryImpl>>,
) -> Result<Redirect, HttpError> {
    let auth_url = oauth2_usecase.authenticate().await?;

    Ok(Redirect::to(auth_url.as_str()))
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    code: String,
    state: CsrfToken,
}

#[tracing::instrument(skip(oauth2_usecase, code, csrf_token))]
async fn discord_auth_callback(
    State(oauth2_usecase): State<OAuth2UseCase<MemberDataRepositoryImpl, OAuth2RepositoryImpl>>,
    Query(AuthRequest {
        code,
        state: csrf_token,
    }): Query<AuthRequest>,
) -> Result<String, HttpError> {
    oauth2_usecase
        .get_token_data(csrf_token.secret().to_owned(), code)
        .await?;
    Ok("Successed to connect your discord account".to_string())
}
