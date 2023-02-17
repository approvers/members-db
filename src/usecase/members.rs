use crate::infra::repository::MemberDataRepository;
use crate::model::MemberDataRow;
use anyhow::Context as _;

#[derive(Clone)]
pub(crate) struct MembersUseCase<R: Clone> {
    member_data_repository: R,
}

impl<R: MemberDataRepository + Clone> MembersUseCase<R> {
    pub(crate) fn new(member_data_repository: R) -> Self {
        Self {
            member_data_repository,
        }
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

    #[tracing::instrument(skip(self))]
    pub(crate) async fn get_all_members(&self) -> anyhow::Result<Vec<MemberDataRow>> {
        self.member_data_repository
            .get_all_members()
            .await
            .context("could not get members data from database")
    }
}
