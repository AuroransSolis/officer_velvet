use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

#[command]
pub async fn source(ctx: &Context, message: &Message) -> CommandResult {
    let _ = message
        .reply(
            &ctx.http,
            "My source code is available at https://github.com/AuroransSolis/officer_velvet.",
        )
        .await?;
    Ok(())
}
