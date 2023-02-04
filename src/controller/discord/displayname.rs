use anyhow::Context as _;
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;

use crate::usecase::firebase::FirebaseUseCaseContainer;

#[group]
#[prefixes("displayname")]
#[summary = "表示名関連コマンド"]
#[description = "members-db APIでの表示名を操作するコマンド"]
#[commands(set_display_name, unset_display_name)]
pub(crate) struct DisplayName;

#[command("set")]
#[description = "表示名を指定した文字列に変更する"]
async fn set_display_name(ctx: &Context, message: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let usecases = data
        .get::<FirebaseUseCaseContainer>()
        .context("could not get usecase container from serenity context")?;

    let Ok(new_display_name) = args
        .single_quoted::<String>() else {
            message
            .reply(
                ctx,
                "表示名を入力してください"
            )
            .await?;
            tracing::info!(
                "could not get new display name from argument: userId: {id}",
                id = message.author.id.to_string(),
            );
            return Ok(());
        };

    if usecases
        .members
        .update_member_display_name(message.author.id.to_string(), new_display_name.clone())
        .await
        .is_ok()
    {
        message
            .reply(
                ctx,
                format!("API上の表示名を{new_display_name}に変更しました"),
            )
            .await?;
        tracing::info!(
            "updated user display name: userId: {id}, displayName: {displayName}",
            id = message.author.id.to_string(),
            displayName = new_display_name
        );
    } else {
        message
            .reply(
                ctx,
                "メンバー情報が見つかりませんでした. 先にOAuth2にて認可を与えてください.",
            )
            .await?;
        tracing::info!(
            "could not get member data: userId: {id}",
            id = message.author.id.to_string(),
        );
    }

    Ok(())
}

#[command("unset")]
#[description = "表示名をデフォルトにリセットする"]
async fn unset_display_name(ctx: &Context, message: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let usecases = data
        .get::<FirebaseUseCaseContainer>()
        .context("could not get usecase container from serenity context")?;

    usecases
        .members
        .unset_member_display_name(message.author.id.to_string())
        .await?;

    message
        .reply(ctx, "API上の表示名をデフォルトにリセットしました")
        .await?;

    tracing::info!(
        "unset user display name: userId: {id}",
        id = message.author.id.to_string(),
    );
    Ok(())
}
