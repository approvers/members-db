use std::collections::HashSet;

use async_trait::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::framework::standard::macros::{help, hook};
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, DispatchError, HelpOptions,
};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::UserId;

pub(super) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        tracing::info!("Loggined as the bot user '{}'!", ready.user.name);
    }
}

#[help]
#[individual_command_tip = "各コマンドの詳細なヘルプを見るには、このコマンドの引数にそのコマンド名を与えてください."]
#[command_not_found_text = "{}というコマンドが見つかりませんでした"]
#[guild_only_text = "このコマンドはギルド内でしか使用することができません"]
#[strikethrough_commands_tip_in_dm = "パーミッション、ロール、チャンネル制限などの条件を満たしておらず実行できないコマンドは ~~`取り消し線`~~ で表示されています"]
#[strikethrough_commands_tip_in_guild = "パーミッション、ロール、チャンネル制限などの条件を満たしておらず実行できないコマンドは ~~`取り消し線`~~ で表示されています"]
pub(super) async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await?;
    Ok(())
}

#[hook]
pub(super) async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    tracing::info!(
        "Got command '{}' by user '{}'",
        command_name,
        msg.author.name
    );

    true
}

#[hook]
pub(super) async fn after(
    _ctx: &Context,
    _msg: &Message,
    command_name: &str,
    command_result: CommandResult,
) {
    match command_result {
        Ok(()) => tracing::info!("Processed command '{}'", command_name),
        Err(why) => tracing::error!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
pub(super) async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    tracing::debug!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
pub(super) async fn dispatch_error(
    ctx: &Context,
    msg: &Message,
    error: DispatchError,
    command_name: &str,
) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            if let Err(err) = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again in {} seconds.", info.as_secs()),
                )
                .await
            {
                tracing::error_span!(
                    "dispatch error and could not send error message",
                    command_name = command_name,
                    user = msg.author.id.0
                );
            }
        }
    }
    tracing::info_span!("dispatch error", command_name = command_name);
}
