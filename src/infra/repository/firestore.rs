use anyhow::Context as _;
use async_trait::async_trait;
use firestore::errors::FirestoreError;
use firestore::{paths, struct_path, FirestoreDb};
use tokio::sync::Mutex;

use crate::model::UserData;
use crate::usecase::members::MembersUseCase;
use crate::usecase::UseCaseContainer;

use super::{RepositoryError, UserDataRepository};

pub(crate) struct UserDataRepositoryImpl {
    pub(self) db: Mutex<FirestoreDb>,
    pub(self) collection_name: &'static str,
}

impl UserDataRepositoryImpl {
    pub(crate) fn new(db: FirestoreDb, collection_name: &'static str) -> Self {
        Self {
            db: Mutex::new(db),
            collection_name,
        }
    }
}

#[async_trait]
impl UserDataRepository for UserDataRepositoryImpl {
    #[tracing::instrument(skip(self))]
    async fn save_display_name(
        &self,
        discord_user_id: String,
        new_display_name: Option<String>,
    ) -> Result<(), RepositoryError> {
        let db = self.db.lock().await;
        let mut transaction = db.begin_transaction().await?;

        let mut user_data: UserData = db
            .fluent()
            .select()
            .by_id_in(self.collection_name)
            .obj()
            .one(&discord_user_id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound {
                discord_user_id: discord_user_id.clone(),
            })?;

        user_data.display_name = new_display_name;

        db.fluent()
            .update()
            .fields(paths!(UserData::display_name))
            .in_col(self.collection_name)
            .document_id(&discord_user_id)
            .object(&user_data)
            .add_to_transaction(&mut transaction)?;

        transaction.commit().await?;
        Ok(())
    }
}

impl From<FirestoreError> for RepositoryError {
    fn from(value: FirestoreError) -> Self {
        match value {
            FirestoreError::ErrorInTransaction(err) => Self::TransactionError(Box::new(err)),
            _ => Self::InternalError(Box::new(value)),
        }
    }
}
