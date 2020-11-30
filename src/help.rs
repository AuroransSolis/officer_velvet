use crate::{gulag::GulagApp, misc::is_administrator, tasks::CreateTaskType};
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
    let icon_url = ctx.http.get_current_user().await?.avatar_url().unwrap();
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
                    .field("=>source", "Sends a link to my code repository.", false)
                    .footer(|footer| {
                        footer
                            .text("Your friendly, neighbourhood gulag officer, Officer Velvet")
                            .icon_url(&icon_url)
                    });
                if is_administrator {
                    println!("HL | CE | User is administrator - adding admin commands to embed.");
                    let apps = [
                        ("=>gulag", GulagApp::clap()),
                        ("=>create_task", CreateTaskType::clap()),
                    ];
                    for (cmd, app) in apps.iter() {
                        let mut help_string = vec![b'`'; 3];
                        app.write_help(&mut help_string).unwrap();
                        help_string.extend_from_slice(&[b'`'; 3]);
                        embed.field(cmd, String::from_utf8(help_string).unwrap(), false);
                    }
                    embed.field("=>current_gulags", "Shows a listing of the current gulag sentences.", false);
                }
                println!("HL | CE | Created embed.");
                embed
            })
        })
        .await?;
    println!("HL | Elapsed: {:?}", start.elapsed());
    Ok(())
}
