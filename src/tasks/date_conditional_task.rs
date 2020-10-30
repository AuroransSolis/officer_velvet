use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serenity::model::id::ChannelId;
use std::cmp::PartialEq;

fn time_eq_to_mins(t1: NaiveTime, t2: NaiveTime) -> bool {
    t1.hour() == t2.hour() && t1.minute() == t2.minute()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DateCondition {
    pub time: Option<NaiveTime>,
    pub weekday: Option<Weekday>,
    pub day_of_month: Option<u32>,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DateConditionalTask {
    SendMessage {
        send_to: ChannelId,
        is_embed: bool,
        header: Option<String>,
        header_text: Option<String>,
        content: String,
        footer: Option<String>,
        footer_text: Option<String>,
        upload_file: Option<String>,
        condition: DateCondition,
    },
    UpdateAppearance {
        new_name: String,
        new_icon_url: String,
        condition: DateCondition,
    },
}

impl DateConditionalTask {
    pub fn time_to_act(&self) -> bool {
        match self {
            DateConditionalTask::SendMessage { condition, .. }
            | DateConditionalTask::UpdateAppearance { condition, .. } => condition == Utc::now(),
        }
    }
}
