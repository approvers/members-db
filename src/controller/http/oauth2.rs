use std::sync::{Arc, Mutex};

use anyhow::Context as _;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::Router;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;

use crate::util::safe_env;

use super::HttpError;

struct OAuth2Context {
    csrf_token: CsrfToken,
    pkce_verifier: PkceCodeVerifier,
}

pub(crate) fn route() -> anyhow::Result<Router> {
    let store = Arc::new(Mutex::new(Vec::<OAuth2Context>::new()));
    let oauth2_client = Arc::new(oauth2_client()?);
    let oauth2_state = OAuth2State {
        store,
        oauth2_client,
    };

    Ok(Router::new()
        .route("/discord", get(discord_auth))
        .route("/discord/callback", get(discord_auth_callback))
        .with_state(oauth2_state))
}

#[derive(Clone)]
struct OAuth2State {
    store: Arc<Mutex<Vec<OAuth2Context>>>,
    oauth2_client: Arc<BasicClient>,
}

fn oauth2_client() -> anyhow::Result<BasicClient> {
    let client_id = safe_env("OAUTH2_CLIENT_ID")?;
    let client_secret = safe_env("OAUTH2_CLIENT_SECRET")?;
    let redirect_url = "http://localhost:8080/oauth2/discord/callback".to_string();
    let auth_url = "https://discord.com/api/oauth2/authorize?response_type=code".to_string();
    let token_url = "https://discord.com/api/oauth2/token".to_string();

    Ok(BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(auth_url).context("could not parse oauth2 auth-url")?,
        Some(TokenUrl::new(token_url).context("could not parse oauth2 token-url")?),
    )
    .set_redirect_uri(
        RedirectUrl::new(redirect_url).context("could not parse oauth2 redirect-url")?,
    ))
}

async fn discord_auth(
    State(OAuth2State {
        store,
        oauth2_client,
    }): State<OAuth2State>,
) -> impl IntoResponse {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = oauth2_client
        .authorize_url(CsrfToken::new_random)
        .add_scopes([
            Scope::new("identify".to_string()),
            Scope::new("connections".to_string()),
        ])
        .set_pkce_challenge(pkce_challenge)
        .url();

    let context = OAuth2Context {
        csrf_token,
        pkce_verifier,
    };
    store
        .lock()
        .expect("could not lock the oauth2-context store")
        .push(context);

    Redirect::to(auth_url.as_str())
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    code: String,
    state: CsrfToken,
}

async fn discord_auth_callback(
    Query(AuthRequest {
        code,
        state: csrf_token,
    }): Query<AuthRequest>,
    State(OAuth2State {
        store,
        oauth2_client,
    }): State<OAuth2State>,
) -> Result<(), HttpError> {
    let pkce_verifier = {
        let mut store = store.lock().map_err(|err| {
            tracing::info!("could not lock oauth2-context store: {error}", error = err);
            anyhow::anyhow!("could not lock oauth2-context store")
        })?;
        let Some(position) = store
            .iter()
            .position(|x| x.csrf_token.secret() == csrf_token.secret())
        else {
            tracing::info!("could not find valid scrf-token from database");
            return Err(HttpError(anyhow::anyhow!("could not find valid csrf-token from database")));
        };
        store.remove(position).pkce_verifier
    };

    let token_result = oauth2_client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await;

    let Ok(token) = token_result else {
        return Err(HttpError(anyhow::anyhow!("could not exchange token from token-server")))
    };

    todo!();

    Ok(())
}
