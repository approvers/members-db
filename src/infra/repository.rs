pub(crate) mod firestore;

use async_trait::async_trait;
use thiserror::Error;

use crate::model::CsrfTokenData;

#[async_trait]
pub(crate) trait MemberDataRepository {
    async fn save_oauth2_token(
        &self,
        discord_user_id: String,
        oauth2_access_token: String,
        oauth2_refresh_token: String,
    ) -> Result<(), RepositoryError>;

    async fn save_display_name(
        &self,
        discord_user_id: String,
        new_display_name: Option<String>,
    ) -> Result<(), RepositoryError>;
}

#[async_trait]
pub(crate) trait OAuth2Repository {
    async fn save_csrf_token(
        &self,
        csrf_token: String,
        pkce_verifier: String,
    ) -> Result<(), RepositoryError>;

    async fn delete_csrf_token(&self, csrf_token: String)
        -> Result<CsrfTokenData, RepositoryError>;
}

#[derive(Debug, Error)]
pub(crate) enum RepositoryError {
    #[error("could not find the row from the database. id: {id}")]
    NotFound { id: String },
    #[error("could not begin transaction")]
    TransactionError(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("internal error")]
    InternalError(#[source] Box<dyn std::error::Error + Send + Sync>),
}
