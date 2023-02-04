use std::sync::Arc;

use async_trait::async_trait;
use firestore::{paths, struct_path, FirestoreDb};
use tokio::sync::Mutex;

use crate::infra::repository::{MemberDataRepository, RepositoryError};
use crate::model::{MemberData, MemberOAuth2Data};

#[derive(Clone)]
pub(crate) struct MemberDataRepositoryImpl {
    db: Arc<Mutex<FirestoreDb>>,
    collection_name: &'static str,
}

impl MemberDataRepositoryImpl {
    pub(crate) fn new(db: Arc<Mutex<FirestoreDb>>, collection_name: &'static str) -> Self {
        Self {
            db,
            collection_name,
        }
    }
}

#[async_trait]
impl MemberDataRepository for MemberDataRepositoryImpl {
    async fn save_oauth2_token(
        &self,
        discord_user_id: String,
        oauth2_access_token: String,
        oauth2_refresh_token: String,
    ) -> Result<(), RepositoryError> {
        let db = self.db.lock().await;

        let data = MemberData {
            display_name: None,
            oauth2: MemberOAuth2Data {
                access_token: oauth2_access_token,
                refresh_token: oauth2_refresh_token,
            },
        };

        db.fluent()
            .update()
            .fields(paths!(MemberData::oauth2))
            .in_col(self.collection_name)
            .document_id(&discord_user_id)
            .object(&data)
            .execute()
            .await?;

        Ok(())
    }

    async fn save_display_name(
        &self,
        discord_user_id: String,
        new_display_name: Option<String>,
    ) -> Result<(), RepositoryError> {
        let db = self.db.lock().await;
        let mut transaction = db.begin_transaction().await?;

        let mut user_data: MemberData = db
            .fluent()
            .select()
            .by_id_in(self.collection_name)
            .obj()
            .one(&discord_user_id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound {
                id: discord_user_id.clone(),
            })?;

        user_data.display_name = new_display_name;

        db.fluent()
            .update()
            .fields(paths!(MemberData::display_name))
            .in_col(self.collection_name)
            .document_id(&discord_user_id)
            .object(&user_data)
            .add_to_transaction(&mut transaction)?;

        transaction.commit().await?;
        Ok(())
    }
}
