use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use std::time::Instant;

#[command]
pub async fn source(ctx: &Context, message: &Message) -> CommandResult {
    let start = Instant::now();
    println!("SC | Responding to source command.");
    let _ = message
        .reply(
            &ctx.http,
            "My source code is available at https://github.com/AuroransSolis/officer_velvet.",
        )
        .await?;
    println!("SC | Elapsed: {:?}", start.elapsed());
    Ok(())
}
