use super::task::Task;
use anyhow::Result as AnyResult;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serenity::{
    http::client::Http,
    prelude::{RwLock, TypeMap},
};
use std::{cmp::PartialEq, sync::Arc};

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
pub struct DateConditionalTask {
    task: Task,
    condition: DateCondition,
}

impl DateConditionalTask {
    pub fn time_to_act(&self) -> bool {
        &self.condition == Utc::now()
    }

    pub async fn act(&self, data: &Arc<RwLock<TypeMap>>, http: &Arc<Http>) -> AnyResult<()> {
        self.task.act(data, http).await
    }
}
