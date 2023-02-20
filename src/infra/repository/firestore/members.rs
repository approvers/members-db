use std::sync::Arc;

use async_trait::async_trait;
use firestore::{paths, struct_path, FirestoreDb};
use futures_util::StreamExt as _;
use tokio::sync::Mutex;

use crate::infra::repository::{MemberDataRepository, RepositoryError};
use crate::model::{MemberDataRow, MemberOAuth2Data};

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

        let data = MemberDataRow {
            discord_user_id: discord_user_id.clone(),
            display_name: None,
            oauth2: MemberOAuth2Data {
                access_token: oauth2_access_token,
                refresh_token: oauth2_refresh_token,
            },
        };

        db.fluent()
            .update()
            .fields(paths!(MemberDataRow::{discord_user_id, oauth2}))
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

        let mut user_data: MemberDataRow = db
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
            .fields(paths!(MemberDataRow::display_name))
            .in_col(self.collection_name)
            .document_id(&discord_user_id)
            .object(&user_data)
            .add_to_transaction(&mut transaction)?;

        transaction.commit().await?;
        Ok(())
    }

    async fn get_member(&self, discord_user_id: &str) -> Result<MemberDataRow, RepositoryError> {
        let db = self.db.lock().await;

        db.fluent()
            .select()
            .by_id_in(self.collection_name)
            .obj()
            .one(discord_user_id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound {
                id: discord_user_id.to_owned(),
            })
    }

    async fn get_all_members(&self) -> Result<Vec<MemberDataRow>, RepositoryError> {
        let db = self.db.lock().await;

        let member_data: Vec<MemberDataRow> = db
            .fluent()
            .list()
            .from(self.collection_name)
            .obj()
            .stream_all()
            .await?
            .collect()
            .await;

        Ok(member_data)
    }
}
