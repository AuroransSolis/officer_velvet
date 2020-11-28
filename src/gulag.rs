use crate::{
    cache_keys::{BotIdKey, ConfigKey, TaskSenderKey, TasksKey},
    misc::{insufficient_perms, is_administrator},
    tasks::{gulag::Gulag, TaskType},
};
use anyhow::Result as AnyResult;
use chrono::{prelude::*, Duration};
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{channel::Message, id::UserId},
    prelude::*,
};
use std::{
    io::{Error as IoError, ErrorKind},
    time::Instant,
};
use structopt::{clap::AppSettings, StructOpt};

fn parse_user_id_from_mention(s: &str) -> AnyResult<UserId> {
    println!("GL | ID | Parsing user ID from '{}'", s);
    let trimmed = s.trim_start_matches("<@!").trim_end_matches(">").trim();
    println!("GL | ID | Trimmed to: '{}'", trimmed);
    let raw_id = trimmed.parse::<u64>()?;
    Ok(raw_id.into())
}

#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "Gulag",
    about = "Sends a user to gulag",
    author = "Aurorans Solis",
    settings(&[AppSettings::ColorNever, AppSettings::NoBinaryName])
)]
pub(crate) struct GulagApp {
    #[structopt(
        short = "u",
        long = "user",
        name = "user_id",
        parse(try_from_str = parse_user_id_from_mention),
    )]
    user_id: UserId,
    #[structopt(
        short = "e",
        long = "end",
        name = "end_date",
        required_unless_one(&[
            "duration_secs",
            "duration_mins",
            "duration_hours",
            "duration_days",
            "duration_weeks"
        ]),
    )]
    end_date: Option<DateTime<Utc>>,
    #[structopt(
        short = "s",
        long = "secs",
        name = "duration_secs",
        required_unless_one(&["duration_mins", "duration_hours", "duration_days", "duration_weeks"]),
        conflicts_with("end_date"),
    )]
    duration_secs: Option<i64>,
    #[structopt(
        short = "m",
        long = "mins",
        name = "duration_mins",
        required_unless_one(&["duration_secs", "duration_hours", "duration_days", "duration_weeks"]),
        conflicts_with("end_date"),
    )]
    duration_mins: Option<i64>,
    #[structopt(
        short = "h",
        long = "hours",
        name = "duration_hours",
        required_unless_one(&["duration_secs", "duration_mins", "duration_days", "duration_weeks"]),
        conflicts_with("end_date"),
    )]
    duration_hours: Option<i64>,
    #[structopt(
        short = "d",
        long = "days",
        name = "duration_days",
        required_unless_one(&["duration_secs", "duration_mins", "duration_hours", "duration_weeks"]),
        conflicts_with("end_date"),
    )]
    duration_days: Option<i64>,
    #[structopt(
        short = "w",
        long = "weeks",
        name = "duration_weeks",
        required_unless_one(&["duration_secs", "duration_mins", "duration_hours", "duration_days"]),
        conflicts_with("end_date"),
    )]
    duration_weeks: Option<i64>,
}

fn try_get_gulag(s: &str) -> AnyResult<(UserId, DateTime<Utc>)> {
    println!("GL | Parsing gulag command use from '{}'", s);
    let trimmed = s.trim_start_matches("=>gulag").trim();
    let arg_matches = GulagApp::from_iter_safe(trimmed.split_whitespace())?;
    println!("GL | Successfully parsed usage.");
    let GulagApp {
        user_id,
        end_date,
        duration_secs,
        duration_mins,
        duration_hours,
        duration_days,
        duration_weeks,
    } = arg_matches;
    let end = if let Some(end) = end_date {
        println!("GL | DR | Duration specified by end time.");
        end
    } else {
        println!("GL | DR | Duration specified by parts.");
        let now = Utc::now();
        let constructors_and_values: [(fn(i64) -> Duration, Option<i64>); 5] = [
            (Duration::seconds, duration_secs),
            (Duration::minutes, duration_mins),
            (Duration::hours, duration_hours),
            (Duration::days, duration_days),
            (Duration::weeks, duration_weeks),
        ];
        let duration = constructors_and_values
            .iter()
            .map(|&(constructor, value)| constructor(value.unwrap_or(0)))
            .fold(Duration::zero(), |acc, new| acc + new);
        println!("GL | DR | Summed parts of duration to {}", duration);
        println!("GL | DR | Adding duration to present time UTC.");
        now.checked_add_signed(duration).ok_or_else(|| {
            let msg = format!(
                "GL | DR | Duration ({}) causes overflow when used as an offset from present time.",
                duration
            );
            IoError::new(ErrorKind::InvalidInput, msg.as_str())
        })?
    };
    println!("GL | Successfully parsed user ID and gulag duration.");
    Ok((user_id, end))
}

#[command]
pub async fn gulag(ctx: &Context, message: &Message) -> CommandResult {
    println!("CM | Start handling gulag command.");
    let start = Instant::now();
    println!("GL | Grabbing read 'lock' on context data.");
    let context_data = ctx.data.read().await;
    let self_id = *context_data.get::<BotIdKey>().unwrap();
    println!("GL | Checking permissions.");
    if is_administrator(&ctx.http, context_data, &message).await? {
        match try_get_gulag(message.content.as_str()) {
            Ok((user_id, end)) => {
                if user_id != self_id {
                    println!("GL | Getting write lock on context data.");
                    let mut context_data = ctx.data.write().await;
                    println!("GL | Getting tasks list.");
                    let tasks = context_data.get_mut::<TasksKey>().unwrap();
                    println!(
                        "GL | Checking for existing gulag entries for user ID {}",
                        user_id
                    );
                    // Check if any gulags exist for this user presently, and if they do, update the
                    // end time.
                    if let Some(gulag) = tasks.iter_mut().find_map(|task| match task {
                        TaskType::Gulag(gulag) if gulag.user.1 == user_id => Some(gulag),
                        _ => None,
                    }) {
                        println!("GL | Found existing gulag entry - updating.");
                        gulag.end = end;
                    } else {
                        println!("GL | No gulag entries for that user exist.");
                        println!("GL | Getting guild ID.");
                        let guild_id = context_data.get::<ConfigKey>().unwrap().guild_id;
                        println!("GL | Getting member information.");
                        let mut member =
                            match ctx.http.get_member(guild_id.into(), user_id.into()).await {
                                Ok(member) => Ok(member),
                                Err(err) => {
                                    println!(
                                        "GL | Failed to get member information. Notifying user."
                                    );
                                    let content = format!(
                                        "Failed to get member information. Error details:\n{}",
                                        err
                                    );
                                    let _ = message.reply(&ctx.http, content.as_str()).await?;
                                    Err(err)
                                }
                            }?;
                        println!(
                            "GL | Successfully retrieved member information for '{}' (ID {})",
                            member.display_name(),
                            member.user.id,
                        );
                        let user = (member.display_name().clone().into_owned(), user_id);
                        println!("GL | Fetching guild information.");
                        let guild = match ctx.http.get_guild(guild_id.into()).await {
                            Ok(guild) => Ok(guild),
                            Err(err) => {
                                println!("GL | Failed to get guild information. Notifying user.");
                                let content = format!(
                                    "Failed to fetch guild information to save roles. Details:\n{}",
                                    err
                                );
                                let _ = message.reply(&ctx.http, content.as_str()).await?;
                                Err(err)
                            }
                        }?;
                        println!("GL | Successfully retrieved guild information.");
                        let roles_map = guild.roles;
                        println!("GL | Mapping user IDs to role names.");
                        let roles = member
                            .roles
                            .iter()
                            .map(|&role_id| {
                                (roles_map.get(&role_id).unwrap().name.clone(), role_id)
                            })
                            .collect::<Vec<_>>();
                        println!("GL | Creating gulag entry.");
                        let gulag = Gulag::new(user, roles, end);
                        println!("GL | Getting task sender.");
                        let task_sender = context_data.get_mut::<TaskSenderKey>().unwrap();
                        println!("GL | Sending task to main thread.");
                        match task_sender.send(TaskType::Gulag(gulag)) {
                            Ok(_) => Ok(()),
                            Err(err) => {
                                println!(
                                    "GL | SN | Failed to send task to main thread. Notifying user."
                                );
                                let content = format!(
                                    "Failed to send gulag task to task handler. Details:\n{}",
                                    err
                                );
                                let _ = message.reply(&ctx.http, content.as_str()).await?;
                                Err(err)
                            }
                        }?;
                        println!("GL | SN | Successfully sent task to main thread.");
                        println!("GL | Getting gulag role ID.");
                        let gulag_id = context_data.get::<ConfigKey>().unwrap().prisoner_role_id;
                        println!("GL | Removing user's roles.");
                        let roles = member.roles.clone();
                        member.remove_roles(&ctx.http, &roles).await?;
                        println!("GL | Adding prisoner role.");
                        ctx.http
                            .add_member_role(guild_id.into(), user_id.into(), gulag_id.into())
                            .await?;
                        println!("GL | Successfully gulagged user.");
                    }
                }
            }
            Err(err) => {
                println!("GL | User input an invalid command. Displaying error message.");
                let content = format!("Error parsing command. Details:\n```{}\n```", err);
                let _ = message
                    .channel_id
                    .send_message(&ctx.http, |m| m.content(content.as_str()))
                    .await?;
            }
        }
    } else {
        insufficient_perms(ctx, message).await?;
    }
    println!("GL | Elapsed: {:?}", start.elapsed());
    Ok(())
}
