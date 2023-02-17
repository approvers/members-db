use crate::infra::repository::{MemberDataRepository, RepositoryError};
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
            .context("error occurred when updating user display name")
            .inspect_err(|err| tracing::error!("{}", err))?;
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
            .context("error occurred when updating user display name")
            .inspect_err(|err| tracing::error!("{}", err))?;
        tracing::info!("updated member display name to default");

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn get_all_members(&self) -> anyhow::Result<Vec<MemberDataRow>> {
        self.member_data_repository
            .get_all_members()
            .await
            .context("could not get members data from database")
            .inspect_err(|err| tracing::error!("{}", err))
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn get_member(
        &self,
        discord_user_id: &str,
    ) -> anyhow::Result<Option<MemberDataRow>> {
        let member = self
            .member_data_repository
            .get_member(discord_user_id)
            .await;
        match member {
            Ok(member) => Ok(Some(member)),
            Err(err) => match err {
                RepositoryError::NotFound { .. } => Ok(None),
                _ => Err(err.into()),
            },
        }
    }
}
