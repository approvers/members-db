use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use firestore::{paths, struct_path, FirestoreDb};
use tokio::sync::Mutex;

use crate::infra::repository::{OAuth2Repository, RepositoryError};
use crate::model::CsrfTokenData;

#[derive(Clone)]
pub(crate) struct OAuth2RepositoryImpl {
    pub(self) db: Arc<Mutex<FirestoreDb>>,
    pub(self) collection_name: &'static str,
}

impl OAuth2RepositoryImpl {
    pub(crate) fn new(db: Arc<Mutex<FirestoreDb>>, collection_name: &'static str) -> Self {
        Self {
            db,
            collection_name,
        }
    }
}

#[async_trait]
impl OAuth2Repository for OAuth2RepositoryImpl {
    async fn save_csrf_token(
        &self,
        csrf_token: String,
        pkce_verifier: String,
    ) -> Result<(), RepositoryError> {
        let db = self.db.lock().await;

        let data = CsrfTokenData {
            pkce_verifier,
            expires_at: Utc::now() + Duration::days(1),
        };

        db.fluent()
            .update()
            .fields(paths!(CsrfTokenData::{pkce_verifier, expires_at}))
            .in_col(self.collection_name)
            .document_id(&csrf_token)
            .object(&data)
            .execute::<CsrfTokenData>()
            .await?;

        Ok(())
    }

    async fn delete_csrf_token(
        &self,
        csrf_token: String,
    ) -> Result<CsrfTokenData, RepositoryError> {
        let db = self.db.lock().await;
        let mut transaction = db.begin_transaction().await?;

        let data: CsrfTokenData = db
            .fluent()
            .select()
            .by_id_in(self.collection_name)
            .obj()
            .one(&csrf_token)
            .await?
            .ok_or_else(|| RepositoryError::NotFound {
                id: csrf_token.clone(),
            })?;

        db.fluent()
            .delete()
            .from(self.collection_name)
            .document_id(&csrf_token)
            .add_to_transaction(&mut transaction)?;

        transaction.commit().await?;
        Ok(data)
    }
}
