use crate::{cache_keys::ConfigKey, tasks::TaskType};
use anyhow::Result as AnyResult;
use chrono::{DateTime, Duration, Utc};
use clap::{
    error::{DefaultFormatter, Error},
    ColorChoice, Command, Parser,
};
use serenity::{http::CacheHttp, model::channel::Message, prelude::*};
use std::io::{Error as IoError, ErrorKind};
use tokio::{fs::File as AsyncFile, io::AsyncWriteExt, sync::RwLockReadGuard};

// This file just contains some QoL stuff. Nothing important.

pub type ClapResult<T, F = DefaultFormatter> = Result<T, Error<F>>;

macro_rules! debug_println {
    ($($all:tt)*) => {
        #[cfg(debug_assertions)]
        {
            println!($($all)*);
        }
    }
}

pub(crate) use debug_println;

pub fn escape_formatting<S: AsRef<str>>(s: S) -> String {
    s.as_ref().replace(['*', '|', '_', '~', '`'], "\\*")
}

pub async fn is_administrator(
    http: impl CacheHttp,
    context_data: RwLockReadGuard<'_, TypeMap>,
    message: &Message,
) -> AnyResult<bool> {
    println!("CK | Getting user's roles.");
    let user_roles = message.member(http).await?.roles;
    Ok(context_data
        .get::<ConfigKey>()
        .unwrap()
        .elevated_roles
        .iter()
        .any(|(_, role_id)| user_roles.contains(role_id)))
}

pub async fn insufficient_perms(ctx: &Context, message: &Message) -> AnyResult<()> {
    println!("    User has insufficient permissions. Notifying and returning.");
    let _ = message
        .reply(
            &ctx.http,
            "You'd best slide me over a bit of the good-good, comrade, or the officers will hear \
        about your attempt to usurp authority.",
        )
        .await?;
    Ok(())
}

#[derive(Clone, Debug, Parser)]
#[command(
    name = "Create Time Period",
    color(ColorChoice::Never),
    no_binary_name(true)
)]
pub struct CreateTimePeriod {
    #[arg(
        short = 'e',
        long = "end",
        name = "end_date",
        required_unless_present_any(&[
            "duration_secs",
            "duration_mins",
            "duration_hours",
            "duration_days",
            "duration_weeks"
        ]),
    )]
    end_date: Option<DateTime<Utc>>,
    #[arg(
        short = 's',
        long = "secs",
        name = "duration_secs",
        required_unless_present_any(&["duration_mins", "duration_hours", "duration_days", "duration_weeks"]),
        conflicts_with("end_date"),
    )]
    duration_secs: Option<i64>,
    #[arg(
        short = 'm',
        long = "mins",
        name = "duration_mins",
        required_unless_present_any(&["duration_secs", "duration_hours", "duration_days", "duration_weeks"]),
        conflicts_with("end_date"),
    )]
    duration_mins: Option<i64>,
    #[arg(
        short = 'h',
        long = "hours",
        name = "duration_hours",
        required_unless_present_any(&["duration_secs", "duration_mins", "duration_days", "duration_weeks"]),
        conflicts_with("end_date"),
    )]
    duration_hours: Option<i64>,
    #[arg(
        short = 'd',
        long = "days",
        name = "duration_days",
        required_unless_present_any(&["duration_secs", "duration_mins", "duration_hours", "duration_weeks"]),
        conflicts_with("end_date"),
    )]
    duration_days: Option<i64>,
    #[arg(
        short = 'w',
        long = "weeks",
        name = "duration_weeks",
        required_unless_present_any(&["duration_secs", "duration_mins", "duration_hours", "duration_days"]),
        conflicts_with("end_date"),
    )]
    duration_weeks: Option<i64>,
}

type CVPairs<'a> = [(fn(i64) -> Duration, &'a Option<i64>); 5];

impl CreateTimePeriod {
    pub fn to_duration(&self) -> Duration {
        let CreateTimePeriod {
            end_date,
            duration_secs,
            duration_mins,
            duration_hours,
            duration_days,
            duration_weeks,
        } = self;
        if let Some(end) = end_date {
            println!("DR | Duration specified by end time.");
            end.signed_duration_since(Utc::now())
        } else {
            println!("DR | Duration specified by parts.");
            let constructors_and_values: CVPairs = [
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
            println!("DR | Summed parts of duration to {duration}");
            duration
        }
    }

    pub fn to_datetime_utc(&self) -> AnyResult<DateTime<Utc>> {
        let CreateTimePeriod {
            end_date,
            duration_secs,
            duration_mins,
            duration_hours,
            duration_days,
            duration_weeks,
        } = self;
        if let Some(end) = end_date {
            println!("DR | Duration specified by end time.");
            Ok(*end)
        } else {
            println!("DR | Duration specified by parts.");
            let now = Utc::now();
            let constructors_and_values: CVPairs = [
                (Duration::seconds, duration_secs),
                (Duration::minutes, duration_mins),
                (Duration::hours, duration_hours),
                (Duration::days, duration_days),
                (Duration::weeks, duration_weeks),
            ];
            let d = constructors_and_values
                .iter()
                .map(|&(constructor, value)| constructor(value.unwrap_or(0)))
                .fold(Duration::zero(), |acc, new| acc + new);
            println!("DR | Summed parts of duration to {d}");
            println!("DR | Adding duration to present time UTC.");
            now.checked_add_signed(d).ok_or_else(|| {
                let msg = format!(
                    "DR | Duration ({d}) causes overflow when used as an offset from present time."
                );
                IoError::new(ErrorKind::InvalidInput, msg.as_str()).into()
            })
        }
    }
}

pub async fn update_task_list(filename: &str, tasklist: &[TaskType]) -> AnyResult<()> {
    println!("TL | Writing out task list.");
    println!("TL | Assigning context task list to changed list.");
    let mut tasks_file = AsyncFile::create(filename).await?;
    let new_contents = serde_json::to_string_pretty(tasklist).unwrap();
    tasks_file
        .write_all(new_contents.as_bytes())
        .await
        .map_err(Into::into)
}

pub fn get_help_msg(mut app: Command) -> String {
    format!("```{}```", app.render_help())
}

#[cfg(test)]
mod test {
    use super::CreateTimePeriod;
    use chrono::Duration;

    fn test_create_time_period_duration() {
        let ctp = CreateTimePeriod {
            end_date: None,
            duration_secs: Some(100),
            duration_hours: None,
            duration_mins: None,
            duration_days: None,
            duration_weeks: None,
        };
        assert_eq!(ctp.to_duration(), Duration::seconds(100));
    }
}
