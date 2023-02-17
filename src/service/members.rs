use anyhow::Context;
use futures_util::{stream, StreamExt as _};
use serenity::http::Http;
use serenity::model::guild::Role;

use crate::infra::repository::{MemberDataRepository, OAuth2Repository};
use crate::model::{MemberListRow, RoleInfo};
use crate::usecase::members::MembersUseCase;
use crate::usecase::oauth2::OAuth2UseCase;
use crate::util::safe_env;

#[derive(Clone)]
pub(crate) struct MembersService<MR: Clone, OR: Clone> {
    members_usecase: MembersUseCase<MR>,
    oauth2_usecase: OAuth2UseCase<MR, OR>,
    guild_id: u64,
}

impl<MR, OR> MembersService<MR, OR>
where
    MR: Clone + MemberDataRepository,
    OR: Clone + OAuth2Repository,
{
    pub(crate) fn new(
        members_usecase: MembersUseCase<MR>,
        oauth2_usecase: OAuth2UseCase<MR, OR>,
        guild_id: u64,
    ) -> Self {
        Self {
            members_usecase,
            oauth2_usecase,
            guild_id,
        }
    }

    pub(crate) async fn get_all_members(&self) -> anyhow::Result<Vec<MemberListRow>> {
        let members = self.members_usecase.get_all_members().await?;

        stream::iter(members.iter())
            .then(|m| async move {
                let user_access_token = self
                    .oauth2_usecase
                    .refresh_token(&m.discord_user_id)
                    .await?;
                let user_http =
                    Http::new(&format!("Bearer {}", user_access_token.secret().as_str()));
                let bot_http = Http::new(&safe_env("DISCORD_TOKEN")?);

                let connections = user_http
                    .get_user_connections()
                    .await
                    .context("could not fetch user connections from discord oauth2 server")?;
                let highest_role = self
                    .get_highest_role(&bot_http, m.discord_user_id.parse()?)
                    .await;

                Ok(MemberListRow {
                    discord_user_id: m.discord_user_id.to_owned(),
                    display_name: m.display_name.to_owned(),
                    twitter: connections
                        .iter()
                        .filter(|x| x.kind == *"twitter")
                        .map(|x| x.id.to_owned())
                        .collect(),
                    github: connections
                        .iter()
                        .filter(|x| x.kind == *"github")
                        .map(|x| x.id.to_owned())
                        .collect(),
                    role: highest_role.map(|role| RoleInfo {
                        name: role.name.to_owned(),
                        color: role.colour.hex(),
                    }),
                })
            })
            .collect::<Vec<anyhow::Result<MemberListRow>>>()
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()
    }

    #[tracing::instrument(skip(self, http))]
    async fn get_highest_role(&self, http: &Http, member_id: u64) -> Option<Role> {
        let guild_roles = http
            .get_guild_roles(self.guild_id)
            .await
            .inspect(|roles| {
                tracing::debug!("fetched existing guild roles from discord: {:?}", roles)
            })
            .inspect_err(|err| {
                tracing::warn!("could not fetch guild roles from discord: {}", err);
            })
            .ok()?;
        let member = http
            .get_member(self.guild_id, member_id)
            .await
            .inspect_err(|err| tracing::warn!("could not fetch guild member from discord: {}", err))
            .ok()?;

        let mut highest: Option<&Role> = None;

        for role_id in &member.roles {
            let Some(role) = guild_roles.iter().find(|x| x.id == *role_id.as_u64()) else {
                tracing::warn!("could not find role from guilds: guild_id: {}, role_id: {}", self.guild_id, role_id);
                continue;
            };

            if let Some(highest) = highest {
                if role.position < highest.position
                    || (role.position == highest.position && role.id > highest.id)
                {
                    continue;
                }
            }

            highest = Some(role);
        }

        highest.cloned()
    }
}