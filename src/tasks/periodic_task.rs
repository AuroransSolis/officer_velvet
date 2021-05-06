use super::task::Task;
use crate::misc::CreateTimePeriod;
use anyhow::Result as AnyResult;
use chrono::{Duration, prelude::*};
use serde::{Deserialize, Serialize};
use serenity::{
    http::client::Http,
    prelude::{RwLock, TypeMap},
};
use std::{
    io::{Error as IoError, ErrorKind},
    sync::Arc,
};
use structopt::{clap::AppSettings, StructOpt};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeriodicTask {
    pub task: Task,
    pub diff: i64,
    pub last_sent: NaiveDateTime,
}

impl PeriodicTask {
    pub fn elapse_period(&mut self) -> AnyResult<()> {
        self.last_sent = self.last_sent.checked_add_signed(Duration::seconds(self.diff)).ok_or(IoError::new(
            ErrorKind::InvalidData,
            "Advancing a periodic task's dates produced an invalid date.",
        ))?;
        Ok(())
    }

    pub fn time_to_act(&self) -> bool {
        Utc::now().naive_utc().signed_duration_since(self.last_sent) >= Duration::seconds(self.diff)
    }

    pub async fn act(&mut self, data: &Arc<RwLock<TypeMap>>, http: &Arc<Http>) -> AnyResult<()> {
        self.task.act(data, http).await?;
        while Utc::now().naive_utc().signed_duration_since(self.last_sent) >= Duration::seconds(self.diff) {
            self.elapse_period()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "Create Periodic Task",
    settings(&[AppSettings::ColorNever, AppSettings::NoBinaryName]),
)]
pub struct CreatePeriodicTask {
    #[structopt(skip)]
    pub task: Task,
    #[structopt(long = "start", name = "start_sending")]
    pub start: NaiveDateTime,
    #[structopt(flatten)]
    pub duration: CreateTimePeriod,
}

impl CreatePeriodicTask {
    pub fn create(self) -> AnyResult<PeriodicTask> {
        Ok(PeriodicTask {
            task: self.task,
            diff: self.duration.to_duration().num_seconds(),
            last_sent: Utc::now().naive_utc(),
        })
    }
}
