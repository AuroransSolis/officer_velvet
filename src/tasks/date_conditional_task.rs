use super::task::Task;
use anyhow::Result as AnyResult;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serenity::{
    http::client::Http,
    prelude::{RwLock, TypeMap},
};
use std::{
    cmp::PartialEq,
    io::{Error as IoError, ErrorKind},
    sync::Arc,
};
use structopt::{clap::AppSettings, StructOpt};

fn time_eq_to_mins(t1: NaiveTime, t2: NaiveTime) -> bool {
    t1.hour() == t2.hour() && t1.minute() == t2.minute()
}

fn parse_weekday(s: &str) -> AnyResult<Weekday> {
    match s.to_lowercase().as_str() {
        "m" | "mon" | "monday" => Ok(Weekday::Mon),
        "t" | "tue" | "tuesday" => Ok(Weekday::Tue),
        "w" | "wed" | "wednesday" => Ok(Weekday::Wed),
        "th" | "thu" | "thursday" => Ok(Weekday::Thu),
        "f" | "fri" | "friday" => Ok(Weekday::Fri),
        "s" | "sat" | "saturday" => Ok(Weekday::Sat),
        "su" | "sun" | "sunday" => Ok(Weekday::Sun),
        _ => Err(IoError::new(
            ErrorKind::InvalidInput,
            format!("Invalid weekday '{}'", s).as_str(),
        )
        .into()),
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, StructOpt)]
pub struct DateCondition {
    #[structopt(short = "t", long = "time", name = "time")]
    pub time: Option<NaiveTime>,
    #[structopt(
        short = "w",
        long = "weekday",
        name = "weekday",
        conflicts_with("day_of_month"),
        parse(try_from_str = parse_weekday))
    ]
    pub weekday: Option<Weekday>,
    #[structopt(
        short = "d",
        long = "day_of_month",
        visible_alias("dom"),
        name = "day_of_month"
    )]
    pub day_of_month: Option<u32>,
    #[structopt(
        short = "m",
        long = "month_of_year",
        visible_alias("moy"),
        name = "month_of_year"
    )]
    pub month_of_year: Option<u32>,
}

impl<'a> PartialEq<DateTime<Utc>> for &'a DateCondition {
    fn eq(&self, other: &DateTime<Utc>) -> bool {
        self.time
            .map(|time| time_eq_to_mins(time, other.time()))
            .map(|result| {
                self.weekday
                    .map(|weekday| weekday == other.weekday() && result)
                    .unwrap_or(result)
            })
            .map(|result| {
                self.day_of_month
                    .map(|dom| dom == other.day() && result)
                    .unwrap_or(result)
            })
            .map(|result| {
                self.month_of_year
                    .map(|moy| moy == other.month() && result)
                    .unwrap_or(result)
            })
            .unwrap_or(false)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, StructOpt)]
#[structopt(
    name = "Date Conditional Task",
    settings(&[AppSettings::ColorNever, AppSettings::NoBinaryName]),
)]
pub struct DateConditionalTask {
    #[structopt(skip)]
    pub task: Task,
    #[structopt(flatten)]
    pub condition: DateCondition,
}

impl DateConditionalTask {
    pub fn time_to_act(&self) -> bool {
        &self.condition == Utc::now()
    }

    pub async fn act(&self, data: &Arc<RwLock<TypeMap>>, http: &Arc<Http>) -> AnyResult<()> {
        self.task.act(data, http).await
    }
}
