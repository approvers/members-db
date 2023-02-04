use anyhow::Context;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AccessToken, AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RefreshToken,
    Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};

use crate::infra::repository::OAuth2Repository;

#[derive(Clone)]
pub(crate) struct OAuth2UseCase<R: Clone> {
    oauth2_client: BasicClient,
    oauth2_repository: R,
}

impl<R: OAuth2Repository + Clone> OAuth2UseCase<R> {
    pub(crate) fn new(oauth2_client: BasicClient, oauth2_repository: R) -> Self {
        Self {
            oauth2_client,
            oauth2_repository,
        }
    }

    /// Returns auth-url.
    pub(crate) async fn authenticate(&self) -> anyhow::Result<String> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = self
            .oauth2_client
            .authorize_url(CsrfToken::new_random)
            .add_scopes([
                Scope::new("identify".to_string()),
                Scope::new("connections".to_string()),
            ])
            .set_pkce_challenge(pkce_challenge)
            .url();

        self.oauth2_repository
            .save_csrf_token(
                csrf_token.secret().to_owned(),
                pkce_verifier.secret().to_owned(),
            )
            .await
            .context("could not save csrf-token and pkce-verifier")?;

        Ok(auth_url.to_string())
    }

    pub(crate) async fn get_token_data(
        &self,
        csrf_token: String,
        code: String,
    ) -> anyhow::Result<(String, AccessToken, RefreshToken)> {
        let token_data = self
            .oauth2_repository
            .delete_csrf_token(csrf_token)
            .await
            .context("could not get csrf-token from database")?;

        let pkce_verifier = PkceCodeVerifier::new(token_data.pkce_verifier);

        let token = self
            .oauth2_client
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

        Ok((
            user.id,
            token.access_token().to_owned(),
            token
                .refresh_token()
                .context("refresh token is not provided by discord oauth2 server")?
                .to_owned(),
        ))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    avatar: Option<String>,
    username: String,
    discriminator: String,
}
