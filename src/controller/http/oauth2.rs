use anyhow::Context;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::get;
use axum::Router;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};

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
    State(oauth2_client): State<BasicClient>,
    State(oauth2_usecase): State<OAuth2UseCase<OAuth2RepositoryImpl>>,
) -> Result<Redirect, HttpError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = oauth2_client
        .authorize_url(CsrfToken::new_random)
        .add_scopes([
            Scope::new("identify".to_string()),
            Scope::new("connections".to_string()),
        ])
        .set_pkce_challenge(pkce_challenge)
        .url();

    oauth2_usecase
        .save_csrf_token(
            csrf_token.secret().to_owned(),
            pkce_verifier.secret().to_owned(),
        )
        .await?;

    Ok(Redirect::to(auth_url.as_str()))
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    code: String,
    state: CsrfToken,
}

async fn discord_auth_callback(
    State(oauth2_client): State<BasicClient>,
    State(oauth2_usecase): State<OAuth2UseCase<OAuth2RepositoryImpl>>,
    State(members_usecase): State<MembersUseCase<MemberDataRepositoryImpl>>,
    Query(AuthRequest {
        code,
        state: csrf_token,
    }): Query<AuthRequest>,
) -> Result<String, HttpError> {
    let data = oauth2_usecase
        .delete_csrf_token(csrf_token.secret().to_owned())
        .await?;
    let pkce_verifier = PkceCodeVerifier::new(data.pkce_verifier);

    let token = oauth2_client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await?;

    let user = reqwest::Client::new()
        .get("https://discord.com/api/users/@me")
        .bearer_auth(token.access_token().secret())
        .send()
        .await?
        .json::<User>()
        .await?;
    let discord_user_id = user.id;

    members_usecase
        .new_member_data(
            discord_user_id,
            token.access_token().secret().to_owned(),
            token
                .refresh_token()
                .context("refresh token is not provided at discord oauth2 server response")?
                .secret()
                .to_owned(),
        )
        .await?;

    Ok("Successed to connect your discord account".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    avatar: Option<String>,
    username: String,
    discriminator: String,
}
