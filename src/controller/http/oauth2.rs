use std::sync::{Arc, Mutex};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use serde::Deserialize;

struct OAuth2Context {
    csrf_token: CsrfToken,
    pkce_verifier: PkceCodeVerifier,
}

pub(crate) fn route() -> Router {
    let store = Arc::new(Mutex::new(Vec::<OAuth2Context>::new()));
    let oauth2_client = Arc::new(oauth2_client());
    let oauth2_state = OAuth2State {
        store,
        oauth2_client,
    };

    Router::new()
        .route("/discord", get(discord_auth))
        .route("/discord/callback", get(discord_auth_callback))
        .with_state(oauth2_state)
}

fn safe_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("could not get env var '{}'", key))
}

#[derive(Clone)]
struct OAuth2State {
    store: Arc<Mutex<Vec<OAuth2Context>>>,
    oauth2_client: Arc<BasicClient>,
}

fn oauth2_client() -> BasicClient {
    let client_id = safe_env("OAUTH2_CLIENT_ID");
    let client_secret = safe_env("OAUTH2_CLIENT_SECRET");
    let redirect_url = "http://localhost:8080/oauth2/discord/callback".to_string();
    let auth_url = "https://discord.com/api/oauth2/authorize?response_type=code".to_string();
    let token_url = "https://discord.com/api/oauth2/token".to_string();

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(auth_url).expect("could not parse oauth2 auth-url"),
        Some(TokenUrl::new(token_url).expect("could not parse oauth2 token-url")),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).expect("could not parse oauth2 redirect-url"))
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

#[axum_macros::debug_handler]
async fn discord_auth_callback(
    Query(AuthRequest {
        code,
        state: csrf_token,
    }): Query<AuthRequest>,
    State(OAuth2State {
        store,
        oauth2_client,
    }): State<OAuth2State>,
) -> impl IntoResponse {
    let pkce_verifier = {
        let mut store = store.lock().expect("could not lock oauth2-context store");
        let Some(position) = store
            .iter()
            .position(|x| x.csrf_token.secret() == csrf_token.secret())
        else {
            return (StatusCode::BAD_REQUEST, "could not find valid csrf-token");
        };
        store.remove(position).pkce_verifier
    };

    let token_result = oauth2_client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await;

    let Ok(token) = token_result else {
        return (
            StatusCode::BAD_REQUEST,
            "could not exchange token from token-server",
        );
    };
    let client = reqwest::Client::new();
    let connections = client
        .get("https://discord.com/api/users/@me/connections")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .unwrap();
    println!("{}", connections.text().await.unwrap());

    (StatusCode::OK, "Success")
}
