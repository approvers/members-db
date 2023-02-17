use anyhow::Context;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AccessToken, AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RefreshToken,
    Scope, TokenResponse,
};
use serenity::http::Http;

use crate::infra::repository::{MemberDataRepository, OAuth2Repository};

#[derive(Clone)]
pub(crate) struct OAuth2UseCase<MR: Clone, OR: Clone> {
    oauth2_client: BasicClient,
    members_repository: MR,
    oauth2_repository: OR,
}

impl<MR: MemberDataRepository + Clone, OR: OAuth2Repository + Clone> OAuth2UseCase<MR, OR> {
    pub(crate) fn new(
        oauth2_client: BasicClient,
        members_repository: MR,
        oauth2_repository: OR,
    ) -> Self {
        Self {
            oauth2_client,
            members_repository,
            oauth2_repository,
        }
    }

    /// Returns auth-url.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn authenticate(&self) -> anyhow::Result<String> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = self
            .oauth2_client
            .authorize_url(CsrfToken::new_random)
            .add_scopes([
                Scope::new("identify".to_string()),
                Scope::new("guilds".to_string()),
                Scope::new("guilds.members.read".to_string()),
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
            .context("could not save csrf-token and pkce-verifier")
            .inspect_err(|err| tracing::error!("{}", err))?;

        Ok(auth_url.to_string())
    }

    #[tracing::instrument(skip(self, csrf_token, code))]
    pub(crate) async fn get_token_data(
        &self,
        csrf_token: String,
        code: String,
    ) -> anyhow::Result<()> {
        let token_data = self
            .oauth2_repository
            .delete_csrf_token(csrf_token)
            .await
            .context("could not get csrf-token from database")
            .inspect_err(|err| tracing::error!("{}", err))?;
        tracing::info!("fetched csrf-token data from database");

        let pkce_verifier = PkceCodeVerifier::new(token_data.pkce_verifier);

        let token = self
            .oauth2_client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await?;
        tracing::info!("fetched token from Discord OAuth2 server");

        let http = Http::new(&format!("Bearer {}", token.access_token().secret()));
        let user = http
            .get_current_user()
            .await
            .context("could not get current user info")
            .inspect_err(|err| tracing::error!("{}", err))?;

        self.members_repository
            .save_oauth2_token(
                user.id.to_string(),
                token.access_token().secret().to_owned(),
                token
                    .refresh_token()
                    .context("refresh token is not provided by discord oauth2 server")
                    .inspect_err(|err| tracing::error!("{}", err))?
                    .secret()
                    .to_owned(),
            )
            .await
            .context("could not save oauth2 token to database")
            .inspect_err(|err| tracing::error!("{}", err))?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn refresh_token(&self, discord_user_id: &str) -> anyhow::Result<AccessToken> {
        let member = self
            .members_repository
            .get_member(discord_user_id)
            .await
            .context("could not get member data from database")
            .inspect_err(|err| tracing::error!("{}", err))?;
        tracing::info!("fetched member data from database");

        let token = self
            .oauth2_client
            .exchange_refresh_token(&RefreshToken::new(member.oauth2.refresh_token))
            .request_async(async_http_client)
            .await
            .inspect_err(|err| {
                tracing::error!(
                    "could not refresh access-token from discord oauth2 server: {}",
                    err
                )
            })?;
        tracing::info!("refreshed token from discord oauth2 server");

        self.members_repository
            .save_oauth2_token(
                discord_user_id.to_owned(),
                token.access_token().secret().to_owned(),
                token
                    .refresh_token()
                    .context("refresh token is not provided by discord oauth2 server")
                    .inspect_err(|err| tracing::error!("{}", err))?
                    .secret()
                    .to_owned(),
            )
            .await
            .context("could not save oauth2 token to database")
            .inspect_err(|err| tracing::error!("{}", err))?;

        Ok(token.access_token().to_owned())
    }
}
