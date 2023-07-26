use crate::{
    cache_keys::TasksKey,
    misc::{insufficient_perms, is_administrator, ClapResult},
    tasks::TaskType,
};
use clap::{error::ErrorKind, ColorChoice, Parser};
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::{Message, UserId},
    prelude::Context,
};
use std::time::Instant;

#[derive(Clone, Debug, Parser)]
#[command(color(ColorChoice::Never), no_binary_name(true))]
pub(crate) struct ReleaseSearchCriteriumApp {
    #[arg(
        short = 'u',
        long = "user",
        name = "user",
        conflicts_with("index"),
        required_unless_present("index")
    )]
    user: Option<UserId>,
    #[arg(
        short = 'i',
        long = "index",
        name = "index",
        conflicts_with("user"),
        required_unless_present("user")
    )]
    index: Option<usize>,
}

#[derive(Debug)]
enum ReleaseSearchCriterium {
    UserId(UserId),
    Index(usize),
}

fn try_get_release_info(s: &str) -> ClapResult<ReleaseSearchCriterium> {
    println!("RG | Parsing remove gulag info command use from '{s}'");
    let trimmed = s.trim_start_matches("=>release").trim();
    println!("RG | Trimmed: '{trimmed}'");
    let arg_matches = ReleaseSearchCriteriumApp::try_parse_from(trimmed.split_whitespace())?;
    println!("RG | Successfully parsed usage.");
    let ReleaseSearchCriteriumApp { user, index } = arg_matches;
    match (user, index) {
        (Some(user), None) => Ok(ReleaseSearchCriterium::UserId(user)),
        (None, Some(index)) => Ok(ReleaseSearchCriterium::Index(index)),
        _ => unreachable!(),
    }
}

#[command]
pub async fn release(ctx: &Context, message: &Message) -> CommandResult {
    println!("RG | Start handling removal of persistent gulag data.");
    let start = Instant::now();
    println!("RG | Grabbing read 'lock' on context data.");
    let context_data = ctx.data.read().await;
    println!("RG | Checking permissions.");
    if is_administrator(&ctx.http, context_data, message).await? {
        println!("RG | Grabbing write 'lock' on context data.");
        let mut context_data = ctx.data.write().await;
        let rgi = match try_get_release_info(&message.content) {
            Ok(rgi) => rgi,
            Err(err) if err.kind() == ErrorKind::DisplayHelp => {
                println!("RG | User requested help.");
                message.reply(&ctx.http, format!("```{err}```")).await?;
                println!("RG | Elapsed: {:?}", start.elapsed());
                return Ok(());
            }
            Err(err) => {
                println!("RG | Failed to parse user input. Sending error back.");
                message
                    .reply(
                        &ctx.http,
                        format!("Error parsing command. Details:\n```{err}```"),
                    )
                    .await?;
                println!("RG | Elapsed: {:?}", start.elapsed());
                return Err(err.into());
            }
        };
        println!("RG | Gulag search criteria: {rgi:?}");
        println!("RG | Grabbing current tasks.");
        let tasks = context_data.get_mut::<TasksKey>().unwrap();
        let gulag = match rgi {
            ReleaseSearchCriterium::Index(index) => {
                tasks.iter_mut().filter_map(TaskType::gulag_mut).nth(index)
            }
            ReleaseSearchCriterium::UserId(user) => tasks
                .iter_mut()
                .filter_map(TaskType::gulag_mut)
                .find(|gulag| gulag.user.1 == user),
        };
        if let Some(gulag) = gulag {
            println!("RG | Found gulag info: {}", gulag.list_fmt());
            gulag.end = chrono::Utc::now();
            println!("RG | Set gulag end time to now.");
            // Removing this from the task list is handled by the task handler.
        } else {
            println!("RG | No gulag tasks found for the given criterium.");
            message
                .reply(&ctx.http, "No gulag tasks found for the given criteria.")
                .await?;
        }
    } else {
        insufficient_perms(ctx, message).await?;
    }
    println!("RG | Elapsed: {:?}", start.elapsed());
    Ok(())
}
