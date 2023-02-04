use anyhow::Context;

use crate::infra::repository::OAuth2Repository;
use crate::model::CsrfTokenData;

#[derive(Clone)]
pub(crate) struct OAuth2UseCase<R: Clone> {
    oauth2_repository: R,
}

impl<R: OAuth2Repository + Clone> OAuth2UseCase<R> {
    pub(crate) fn new(oauth2_repository: R) -> Self {
        Self { oauth2_repository }
    }

    pub(crate) async fn save_csrf_token(
        &self,
        csrf_token: String,
        pkce_verifier: String,
    ) -> anyhow::Result<()> {
        self.oauth2_repository
            .save_csrf_token(csrf_token, pkce_verifier)
            .await
            .context("could not save csrf-token and pkce-verifier")
    }

    pub(crate) async fn delete_csrf_token(
        &self,
        csrf_token: String,
    ) -> anyhow::Result<CsrfTokenData> {
        self.oauth2_repository
            .delete_csrf_token(csrf_token)
            .await
            .context("could not get csrf-token from database")
    }
}
