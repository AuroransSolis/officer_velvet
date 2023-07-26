use super::task::Task;
use crate::misc::CreateTimePeriod;
use anyhow::Result as AnyResult;
use chrono::{prelude::*, Duration};
use clap::{ColorChoice, Parser};
use serde::{Deserialize, Serialize};
use serenity::{
    http::client::Http,
    prelude::{RwLock, TypeMap},
};
use std::{
    io::{Error as IoError, ErrorKind},
    sync::Arc,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeriodicTask {
    pub task: Task,
    pub diff: i64,
    pub last_sent: NaiveDateTime,
}

impl PeriodicTask {
    pub fn elapse_period(&mut self) -> AnyResult<()> {
        self.last_sent = self
            .last_sent
            .checked_add_signed(Duration::seconds(self.diff))
            .ok_or(IoError::new(
                ErrorKind::InvalidData,
                "Advancing a periodic task's dates produced an invalid date.",
            ))?;
        Ok(())
    }

    pub fn time_to_act(&self) -> bool {
        Utc::now().naive_utc().signed_duration_since(self.last_sent) >= Duration::seconds(self.diff)
    }

    pub async fn act(
        &mut self,
        data: &Arc<RwLock<TypeMap>>,
        http: &impl AsRef<Http>,
    ) -> AnyResult<()> {
        self.task.act(data, http).await?;
        while Utc::now().naive_utc().signed_duration_since(self.last_sent)
            >= Duration::seconds(self.diff)
        {
            self.elapse_period()?;
        }
        Ok(())
    }

    pub fn list_fmt(&self) -> String {
        format!(" PT | {} | Last: {}", self.task.list_fmt(), self.last_sent)
    }
}

#[derive(Clone, Debug, Parser)]
#[command(
    name = "Create Periodic Task",
    color(ColorChoice::Never),
    no_binary_name(true)
)]
pub struct CreatePeriodicTask {
    #[arg(skip)]
    pub task: Task,
    #[arg(long = "start", name = "start_sending")]
    pub start: NaiveDateTime,
    #[command(flatten)]
    pub duration: CreateTimePeriod,
}

impl CreatePeriodicTask {
    pub fn create(self) -> PeriodicTask {
        PeriodicTask {
            task: self.task,
            diff: self.duration.to_duration().num_seconds(),
            last_sent: Utc::now().naive_utc(),
        }
    }
}
