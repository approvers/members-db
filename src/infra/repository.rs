pub(crate) mod firestore;

use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub(crate) trait UserDataRepository {
    async fn save_display_name(
        &self,
        discord_user_id: String,
        new_display_name: Option<String>,
    ) -> Result<(), RepositoryError>;
}

#[derive(Debug, Error)]
pub(crate) enum RepositoryError {
    #[error("could not find the row from the database. discord_user_id: {discord_user_id}")]
    NotFound { discord_user_id: String },
    #[error("could not begin transaction")]
    TransactionError(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("internal error")]
    InternalError(#[source] Box<dyn std::error::Error + Send + Sync>),
}
