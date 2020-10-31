use crate::{
    cache_keys::{ConfigKey, TaskSenderKey, TasksKey},
    misc::{insufficient_perms, is_administrator},
    tasks::{gulag::Gulag, TaskType},
};
use anyhow::Result as AnyResult;
use chrono::{prelude::*, Duration};
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{channel::Message, guild::Member, id::UserId, user::User},
    prelude::*,
};
use std::{
    io::{Error as IoError, ErrorKind},
    time::Instant,
};
use structopt::StructOpt;

fn parse_user_id_from_mention(s: &str) -> AnyResult<UserId> {
    if s.starts_with("<@") && s.ends_with(">") {
        let trimmed = s.trim_start_matches("<@").trim_end_matches('>');
        let raw_id = trimmed.parse::<u64>()?;
        Ok(raw_id.into())
    } else {
        let msg = format!(
            "Argument to `-u`/`--user` must be a mention of another user. Got: '{}'",
            s
        );
        Err(IoError::new(ErrorKind::InvalidInput, msg.as_str()).into())
    }
}

#[derive(Clone, Debug, StructOpt)]
struct GulagApp {
    #[structopt(short = "u", long = "user", parse(try_from_str = parse_user_id_from_mention))]
    user_id: UserId,
    #[structopt(
        short = "e",
        long = "end",
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
        required_unless_one(&["duration_mins", "duration_hours", "duration_days", "duration_weeks"]),
        conflicts_with("end_date"),
    )]
    duration_secs: Option<i64>,
    #[structopt(
        short = "m",
        long = "mins",
        required_unless_one(&["duration_secs", "duration_hours", "duration_days", "duration_weeks"]),
        conflicts_with("end_date"),
    )]
    duration_mins: Option<i64>,
    #[structopt(
        short = "h",
        long = "hours",
        required_unless_one(&["duration_secs", "duration_mins", "duration_days", "duration_weeks"]),
        conflicts_with("end_date"),
    )]
    #[structopt(
        short = "d",
        long = "days",
        required_unless_one(&["duration_secs", "duration_mins", "duration_hours", "duration_weeks"]),
        conflicts_with("end_date"),
    )]
    duration_days: Option<i64>,
    #[structopt(
        short = "w",
        long = "weeks",
        required_unless_one(&["duration_secs", "duration_mins", "duration_hours", "duration_days"]),
        conflicts_with("end_date"),
    )]
    duration_weeks: Option<i64>,
}

fn try_get_gulag(s: &str) -> AnyResult<(UserId, DateTime<Utc>)> {
    let trimmed = s.trim_start_matches("=>gulag").trim();
    let arg_matches = GulagApp::clap().get_matches_from_safe(trimmed.split_whitespace())?;
    let who_to_gulag = arg_matches
        .value_of("user_id")
        .unwrap()
        .parse::<u64>()?
        .into();
    let end = if let Some(end) = arg_matches.value_of("end_date") {
        end.parse()?
    } else {
        let now = Utc::now();
        let constructors_and_options: [(fn(i64) -> Duration, &str); 5] = [
            (Duration::seconds, "duration_secs"),
            (Duration::minutes, "duration_mins"),
            (Duration::hours, "duration_hours"),
            (Duration::days, "duration_days"),
            (Duration::weeks, "duration_weeks"),
        ];
        let mut duration = Duration::zero();
        for &(constructor, option) in constructors_and_options.iter() {
            duration = duration + constructor(arg_matches.value_of(option).unwrap_or("0").parse()?);
        }
        now.checked_add_signed(duration).ok_or_else(|| {
            let msg = format!(
                "Duration ({}) causes overflow when used as an offset from present time.",
                duration
            );
            IoError::new(ErrorKind::InvalidInput, msg.as_str())
        })?
    };
    Ok((who_to_gulag, end))
}

#[command]
pub async fn gulag(ctx: &Context, message: &Message) -> CommandResult {
    println!("Start handling gulag command.");
    let start = Instant::now();
    println!("    Grabbing read 'lock' on context data.");
    let context_data = ctx.data.read().await;
    println!("    Checking permissions.");
    if is_administrator(&ctx.http, context_data, &message).await? {
        match try_get_gulag(message.content.as_str()) {
            Ok((user_id, end)) => {
                let mut context_data = ctx.data.write().await;
                let tasks = context_data.get_mut::<TasksKey>().unwrap();
                // Check if any gulags exist for this user presently, and if they do, update the
                // end time.
                if let Some(gulag) = tasks.iter_mut().find_map(|task| match task {
                    TaskType::Gulag(gulag) if gulag.user.1 == user_id => Some(gulag),
                    _ => None,
                }) {
                    gulag.end = end;
                } else {
                    let guild_id = context_data.get::<ConfigKey>().unwrap().guild_id;
                    let member = match ctx.http.get_member(guild_id.into(), user_id.into()).await {
                        Ok(member) => Ok(member),
                        Err(err) => {
                            let content = format!(
                                "Failed to get member information. Error details:\n{}",
                                err
                            );
                            let _ = message.reply(&ctx.http, content.as_str()).await?;
                            Err(err)
                        }
                    }?;
                    let Member {
                        user: User { name, .. },
                        roles: role_ids,
                        ..
                    } = member;
                    let user = (name, user_id);
                    let guild = match ctx.http.get_guild(guild_id.into()).await {
                        Ok(guild) => Ok(guild),
                        Err(err) => {
                            let content = format!(
                                "Failed to fetch guild information to save roles. Details:\n{}",
                                err
                            );
                            let _ = message.reply(&ctx.http, content.as_str()).await?;
                            Err(err)
                        }
                    }?;
                    let roles_map = guild.roles;
                    let roles = role_ids
                        .into_iter()
                        .map(|role_id| (roles_map.get(&role_id).unwrap().name.clone(), role_id))
                        .collect::<Vec<_>>();
                    let gulag = Gulag::new(user, roles, end);
                    let task_sender = context_data.get_mut::<TaskSenderKey>().unwrap();
                    if let Err(err) = task_sender.send(TaskType::Gulag(gulag)) {
                        let content = format!(
                            "Failed to send gulag task to task handler. Details:\n{}",
                            err
                        );
                        let _ = message.reply(&ctx.http, content.as_str()).await?;
                    }
                }
            }
            Err(err) => {
                println!("    User input an invalid command. Displaying error message.");
                let content = format!("Error parsing command. Details:\n{}", err);
                let _ = message
                    .channel_id
                    .send_message(&ctx.http, |m| m.content(content.as_str()))
                    .await?;
            }
        }
    } else {
        insufficient_perms(ctx, message).await?;
    }
    println!("    Elapsed: {:?}", start.elapsed());
    Ok(())
}

/*command!(Gulag(context, message, args) {
    println!("Commence the gulagging.");
    let start = Instant::now();
    if check_administrator(message.member()) {
        // Get input user ID
        let user_id = parse_arg_ok_or_return!(args, u64, start, 10, message,
            "Failed to parse first argument (user ID)");
        let user_id = UserId(user_id);
        // Collect the duration arguments into a vector
        let mut duration_arguments = Vec::new();
        while let Ok(arg) = args.single::<String>() {
            duration_arguments.push(arg);
        }
        // Sum variable for the total duration
        let mut duration = 0;
        // For each duration argument...
        for duration_arg in duration_arguments {
            // Count the number of characters that are not digits...
            if duration_arg.chars().filter(|&c| (c as u8) < ('0' as u8) || (c as u8) > ('9' as u8))
                .count() > 1 {
                // ...and let the user know that the duration argument is invalid if there's more than
                // one non-digit character in the argument.
                reply_log_return!(start, 10, message,
                    format!("Invalid duration argument: {}", duration_arg).as_str());
            }
            // Collect all the digits in the argument
            let arg_digits = duration_arg.chars()
                .take_while(|&c| (c as u8) >= ('0' as u8) && (c as u8) <= ('9' as u8))
                .collect::<String>();
            println!("    Argument digits: {:?}", arg_digits);
            let arg_val = match arg_digits.parse::<u64>() {
                Ok(val) => val,
                Err(_) => reply_log_return!(start, 10, message,
                    format!("Failed to parse number in argument: {}", duration_arg).as_str())
            };
            // Multiply the digits by the appropriate amount based on the last character and add
            // that to the total
            duration += match duration_arg.chars().rev().next().unwrap() {
                'w' => arg_val * WEEK_AS_SECS,
                'd' => arg_val * DAY_AS_SECS,
                'h' => arg_val * HOUR_AS_SECS,
                'm' => arg_val * MIN_AS_SECS,
                's' => arg_val,
                 _ => {
                    let msg = format!("Invalid time unit specifier in argument: {}", duration_arg);
                    let r = message.reply(msg.as_str())?;
                    println!("    {}", msg);
                    println!("    Elapsed: {:?}", start.elapsed());
                    delete_message_after_delay(r, 10);
                    return Ok(());
                }
            };
        }
        // Attempt to write the gulag file
        if let Some(gulag_entry) = write_gulag_file(duration, user_id, &message) {
            // Attempt to start a new gulag sentence
            if start_new_gulag_sentence(&context, &message, gulag_entry) {
                let r = message.reply("Successfully gulagged user!")?;
                println!("    Success!");
                delete_message_after_delay(r, 10);
            } else {
                // If writing the file but gulagging the user fails, try to delete the gulag file.
                let file_string = format!("{}/{}.gulag", GULAG_DIR, user_id.0);
                if let Err(_) = remove_file(file_string.as_str()) {
                    match AURO_UID.to_user() {
                        Ok(user) => drop(user.direct_message(|m| m.content(file_string.as_str()))),
                        Err(_) => println!("    REMOVE FILE BY HAND: {}", file_string)
                    }
                }
            }
        } else {
            let r = message.reply("Failed to write persistent gulag info. Please try again.")?;
            println!("    Failed to write persistent info to file.");
            println!("    Elapsed: {:?}", start.elapsed());
            delete_message_after_delay(r, 10);
        }
    } else {
        let r = message.reply("You have to be an administrator to do that.")?;
        println!("    Permissions error.");
        delete_message_after_delay(r, 10);
    }
    println!("    Elapsed: {:?}", start.elapsed());
});*/
