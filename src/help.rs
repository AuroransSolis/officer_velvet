use crate::{
    cache_keys::ConfigKey,
    gulag::GulagApp,
    misc::is_administrator,
};
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    utils::Colour,
};
use std::time::Instant;
use structopt::StructOpt;

#[command]
pub async fn help(ctx: &Context, message: &Message) -> CommandResult {
    println!("HL | Start handling help command.");
    let start = Instant::now();
    let is_administrator = is_administrator(&ctx.http, ctx.data.read().await, message).await?;
    let icon_url = ctx
        .data
        .read()
        .await
        .get::<ConfigKey>()
        .unwrap()
        .icon_url
        .clone();
    let _ = message
        .channel_id
        .send_message(&ctx.http, |msg| {
            msg.embed(|embed| {
                println!("HL | CE | Adding fields for all users to embed.");
                embed
                    .title("Help")
                    .description("Available commands and how to use them.")
                    .colour(Colour::from_rgb(243, 44, 115))
                    .field("=>help", "Shows this message.", false)
                    .field(
                        "=>anagram",
                        "Randomly scrambles the rest of the message.",
                        false,
                    )
                    .footer(|footer| {
                        footer
                            .text("Your friendly, neighbourhood gulag officer, Officer Velvet")
                            .icon_url(&icon_url)
                    });
                if is_administrator {
                    println!("HL | CE | User is administrator - adding admin commands to embed.");
                    let mut help_string = Vec::new();
                    GulagApp::clap().write_help(&mut help_string).unwrap();
                    embed.field("=>gulag", String::from_utf8(help_string).unwrap(), false);
                }
                println!("HL | CE | Created embed.");
                embed
            })
        })
        .await?;
    println!("HL | Elapsed: {:?}", start.elapsed());
    Ok(())
}
