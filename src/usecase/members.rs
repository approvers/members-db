use anyhow::Context as _;

use crate::infra::repository::MemberDataRepository;

#[derive(Clone)]
pub(crate) struct MembersUseCase<R: Clone> {
    member_data_repository: R,
}

impl<R: MemberDataRepository + Clone> MembersUseCase<R> {
    pub(crate) fn new(user_data_repository: R) -> Self {
        Self {
            member_data_repository: user_data_repository,
        }
    }

    #[tracing::instrument(skip(self, access_token, refresh_token))]
    pub(crate) async fn new_member_data(
        &self,
        discord_user_id: String,
        access_token: String,
        refresh_token: String,
    ) -> anyhow::Result<()> {
        self.member_data_repository
            .save_oauth2_token(discord_user_id, access_token, refresh_token)
            .await
            .context("error occurred when inserting oauth2 member data")?;
        tracing::info!("inserted new member data with oauth2 credentials");

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn update_member_display_name(
        &self,
        discord_user_id: String,
        new_display_name: String,
    ) -> anyhow::Result<()> {
        self.member_data_repository
            .save_display_name(discord_user_id, Some(new_display_name))
            .await
            .context("error occurred when updating user display name")?;
        tracing::info!("updated member display name");

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn unset_member_display_name(
        &self,
        discord_user_id: String,
    ) -> anyhow::Result<()> {
        self.member_data_repository
            .save_display_name(discord_user_id, None)
            .await
            .context("error occurred when updating user display name")?;
        tracing::info!("updated member display name to default");

        Ok(())
    }
}
