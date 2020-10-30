use anyhow::Result as AnyResult;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serenity::model::id::ChannelId;
use std::io::{Error as IoError, ErrorKind};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PeriodicTask {
    SendMessage {
        send_to: ChannelId,
        is_embed: bool,
        header: Option<String>,
        header_text: Option<String>,
        content: String,
        footer: Option<String>,
        footer_text: Option<String>,
        upload_file: Option<String>,
        last_sent: NaiveDate,
        next_send: NaiveDate,
    },
    UpdateAppearance {
        new_name: String,
        new_icon_url: String,
        last_update: NaiveDate,
        next_update: NaiveDate,
    },
}

impl PeriodicTask {
    pub fn elapse_period(&mut self) -> AnyResult<()> {
        let (last, next) = match self {
            PeriodicTask::SendMessage {
                last_sent: last,
                next_send: next,
                ..
            }
            | PeriodicTask::UpdateAppearance {
                last_update: last,
                next_update: next,
                ..
            } => (last, next),
        };
        let diff = next.signed_duration_since(*last);
        *last = *next;
        *next = next.checked_add_signed(diff).ok_or(IoError::new(
            ErrorKind::InvalidData,
            "Advancing a periodic task's dates produced an invalid date.",
        ))?;
        Ok(())
    }

    pub fn time_to_act(&self) -> bool {
        let next = match self {
            PeriodicTask::SendMessage {
                next_send: next, ..
            }
            | PeriodicTask::UpdateAppearance {
                next_update: next, ..
            } => next,
        };
        Utc::now().naive_utc().date() >= *next
    }
}
