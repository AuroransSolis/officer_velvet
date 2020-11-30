use crate::cache_keys::ConfigKey;
use anyhow::Result as AnyResult;
use chrono::{DateTime, Duration, Utc};
use serenity::{http::CacheHttp, model::channel::Message, prelude::*};
use std::io::{Error as IoError, ErrorKind};
use structopt::{clap::AppSettings, StructOpt};
use tokio::sync::RwLockReadGuard;

// This file just contains some QoL stuff. Nothing important.

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

#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "Create Time Period",
    about = "Converts a ",
    author = "Aurorans Solis",
    settings(&[AppSettings::ColorNever, AppSettings::NoBinaryName])
)]
pub struct CreateTimePeriod {
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
            let constructors_and_values: [(fn(i64) -> Duration, &Option<i64>); 5] = [
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
            println!("DR | Summed parts of duration to {}", duration);
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
            Ok(end.clone())
        } else {
            println!("DR | Duration specified by parts.");
            let now = Utc::now();
            let constructors_and_values: [(fn(i64) -> Duration, &Option<i64>); 5] = [
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
            println!("DR | Summed parts of duration to {}", duration);
            println!("DR | Adding duration to present time UTC.");
            now.checked_add_signed(duration).ok_or_else(|| {
                let msg = format!(
                    "DR | Duration ({}) causes overflow when used as an offset from present time.",
                    duration
                );
                IoError::new(ErrorKind::InvalidInput, msg.as_str()).into()
            })
        }
    }
}
