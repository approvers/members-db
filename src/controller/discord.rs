use std::sync::Arc;

use anyhow::Context as _;
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::gateway::GatewayIntents;
use serenity::Client;

use crate::usecase::firebase::FirebaseUseCaseContainer;
use crate::util::safe_env;

mod displayname;
mod hook;

#[tracing::instrument(skip(usecases))]
pub(crate) async fn start_discord_bot(
    usecases: Arc<FirebaseUseCaseContainer>,
) -> anyhow::Result<()> {
    let token = safe_env("DISCORD_TOKEN")?;
    let http = Http::new(&token);

    let bot_id = http.get_current_user().await?.id;

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("").on_mention(Some(bot_id)))
        .before(hook::before)
        .after(hook::after)
        .unrecognised_command(hook::unknown_command)
        .on_dispatch_error(hook::dispatch_error)
        .help(&hook::HELP)
        .group(&displayname::DISPLAYNAME_GROUP);

    let mut intents = GatewayIntents::default();
    intents.insert(GatewayIntents::GUILD_MESSAGES);

    let mut client = Client::builder(token, intents)
        .event_handler(hook::Handler)
        .framework(framework)
        .type_map_insert::<FirebaseUseCaseContainer>(usecases)
        .await
        .context("could not start discord bot")?;

    client
        .start()
        .await
        .context("could not start discord client")
}
