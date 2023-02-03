use anyhow::Context as _;

use crate::infra::repository::UserDataRepository;

pub(crate) struct MembersUseCase<R> {
    user_data_repository: R,
}

impl<R: UserDataRepository> MembersUseCase<R> {
    pub(crate) fn new(user_data_repository: R) -> Self {
        Self {
            user_data_repository,
        }
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn update_user_display_name(
        &self,
        discord_user_id: String,
        new_display_name: String,
    ) -> anyhow::Result<()> {
        tracing::info!("update_user_display_name");
        self.user_data_repository
            .save_display_name(discord_user_id, Some(new_display_name))
            .await
            .context("error occurred when updating user display name")?;
        tracing::info!("update_user_display_name done");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn unset_user_display_name(
        &self,
        discord_user_id: String,
    ) -> anyhow::Result<()> {
        self.user_data_repository
            .save_display_name(discord_user_id, None)
            .await
            .context("error occurred when updating user display name")
    }
}
