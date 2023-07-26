use super::task::Task;
use anyhow::Result as AnyResult;
use chrono::prelude::*;
use clap::{ColorChoice, Parser};
use serde::{Deserialize, Serialize};
use serenity::{
    http::client::Http,
    prelude::{RwLock, TypeMap},
};
use std::{
    cmp::PartialEq,
    fmt::Write,
    io::{Error as IoError, ErrorKind},
    sync::Arc,
};

fn time_eq_to_secs(t1: NaiveTime, t2: NaiveTime) -> bool {
    t1.hour() == t2.hour() && t1.minute() == t2.minute() && t1.second() == t2.second()
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
            format!("Invalid weekday '{s}'").as_str(),
        )
        .into()),
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Parser)]
pub struct DateCondition {
    #[arg(short = 't', long = "time", name = "time")]
    pub time: Option<NaiveTime>,
    #[arg(
        short = 'w',
        long = "weekday",
        name = "weekday",
        conflicts_with("day_of_month"),
        value_parser = parse_weekday,
    )]
    pub weekday: Option<Weekday>,
    #[arg(
        short = 'd',
        long = "day_of_month",
        visible_alias("dom"),
        name = "day_of_month"
    )]
    pub day_of_month: Option<u32>,
    #[arg(
        short = 'm',
        long = "month_of_year",
        visible_alias("moy"),
        name = "month_of_year"
    )]
    pub month_of_year: Option<u32>,
}

impl<'a> PartialEq<DateTime<Utc>> for &'a DateCondition {
    fn eq(&self, other: &DateTime<Utc>) -> bool {
        let time = self
            .time
            .map_or(true, |time| time_eq_to_secs(time, other.time()));
        let wd = self.weekday.map_or(true, |wd| wd == other.weekday());
        let dom = self.day_of_month.map_or(true, |dom| dom == other.day());
        let moy = self.month_of_year.map_or(true, |moy| moy == other.month());
        time && wd && dom && moy
    }
}

impl DateCondition {
    pub fn list_fmt(&self) -> String {
        let mut out = String::new();
        if let Some(time) = self.time {
            write!(out, "T {time} ").unwrap();
        }
        if let Some(wd) = self.weekday {
            if !out.is_empty() {
                out.push(' ');
            }
            write!(out, "WD {wd}").unwrap();
        }
        if let Some(dom) = self.day_of_month {
            if !out.is_empty() {
                out.push(' ');
            }
            write!(out, "DOM {dom}").unwrap();
        }
        if let Some(moy) = self.month_of_year {
            if !out.is_empty() {
                out.push(' ');
            }
            write!(out, "MOY {moy}").unwrap();
        }
        out
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Parser)]
#[command(
    name = "Date Conditional Task",
    color(ColorChoice::Never),
    no_binary_name(true)
)]
pub struct DateConditionalTask {
    #[arg(skip)]
    pub task: Task,
    #[command(flatten)]
    pub condition: DateCondition,
}

impl DateConditionalTask {
    pub fn time_to_act(&self) -> bool {
        &self.condition == Utc::now()
    }

    pub async fn act(&self, data: &Arc<RwLock<TypeMap>>, http: &impl AsRef<Http>) -> AnyResult<()> {
        self.task.act(data, http).await
    }

    pub fn list_fmt(&self) -> String {
        format!(
            "DCT | {} | {}",
            self.task.list_fmt(),
            self.condition.list_fmt(),
        )
    }
}
