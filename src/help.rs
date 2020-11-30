use crate::{
    gulag::GulagApp,
    misc::is_administrator,
    tasks::{
        CreateTaskType,
        date_conditional_task::DateConditionalTask,
        periodic_task::CreatePeriodicTask
    },
};
use lazy_static::lazy_static;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    utils::Colour,
};
use std::time::Instant;
use structopt::{clap::App, StructOpt};

fn get_help_msg(app: App) -> String {
    let mut help_string = vec![b'`'; 3];
    app.write_help(&mut help_string).unwrap();
    help_string.extend_from_slice(&[b'`'; 3]);
    String::from_utf8(help_string).unwrap()
}

lazy_static! {
    pub static ref GULAG_HELP_MSG: String = get_help_msg(GulagApp::clap());
    pub static ref CREATE_TASK_HELP_MSG: String = {
        let mut string = get_help_msg(CreateTaskType::clap());
        string.push('\n');
        string.push_str(get_help_msg(DateConditionalTask::clap()).as_str());
        string.push('\n');
        string.push_str(get_help_msg(CreatePeriodicTask::clap()).as_str());
        string
    };
}

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
                    let apps = vec![
                        ("=>gulag", GULAG_HELP_MSG.as_str()),
                        ("=>create_task", CREATE_TASK_HELP_MSG.as_str()),
                        ("=>current_gulags", "Show current gulag sentences."),
                    ];
                    for (cmd, desc) in apps.iter() {
                        embed.field(cmd, desc, false);
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
