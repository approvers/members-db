use anyhow::Context as _;
use serenity::http::Http;

pub(crate) struct MembersUseCase {
    guild_id: u64,
    client: Http,
}

impl MembersUseCase {
    pub(crate) async fn get_all_members(&self) -> anyhow::Result<()> {
        let members = self
            .client
            .get_guild_members(self.guild_id, None, None)
            .await
            .context("could not fetch guild members")?;

        Ok(())
    }
}
