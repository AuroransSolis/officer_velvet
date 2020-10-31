use super::task::Task;
use anyhow::Result as AnyResult;
use chrono::prelude::*;
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
    pub last_sent: NaiveDate,
    pub next_send: NaiveDate,
}

impl PeriodicTask {
    pub fn elapse_period(&mut self) -> AnyResult<()> {
        let diff = self.next_send.signed_duration_since(self.last_sent);
        self.last_sent = self.next_send;
        self.next_send = self.next_send.checked_add_signed(diff).ok_or(IoError::new(
            ErrorKind::InvalidData,
            "Advancing a periodic task's dates produced an invalid date.",
        ))?;
        Ok(())
    }

    pub fn time_to_act(&self) -> bool {
        Utc::now().naive_utc().date() >= self.next_send
    }

    pub async fn act(&mut self, data: &Arc<RwLock<TypeMap>>, http: &Arc<Http>) -> AnyResult<()> {
        self.task.act(data, http).await?;
        while self.next_send <= Utc::now().date().naive_utc() {
            self.elapse_period()?;
        }
        Ok(())
    }
}
