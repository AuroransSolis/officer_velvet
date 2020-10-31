pub mod date_conditional_task;
pub mod gulag;
pub mod message;
pub mod periodic_task;
pub mod task;

use anyhow::Result as AnyResult;
use date_conditional_task::DateConditionalTask;
use gulag::Gulag;
use periodic_task::PeriodicTask;
use serde::{Deserialize, Serialize};
use serenity::{
    http::client::Http,
    prelude::{RwLock, TypeMap},
};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TaskType {
    DateConditionalTask(DateConditionalTask),
    Gulag(Gulag),
    PeriodicTask(PeriodicTask),
}

impl TaskType {
    pub fn time_to_act(&self) -> bool {
        match self {
            TaskType::DateConditionalTask(task) => task.time_to_act(),
            TaskType::Gulag(task) => task.time_to_act(),
            TaskType::PeriodicTask(task) => task.time_to_act(),
        }
    }

    pub fn is_gulag(&self) -> bool {
        match self {
            TaskType::Gulag(_) => true,
            _ => false,
        }
    }

    pub async fn act(&mut self, data: &Arc<RwLock<TypeMap>>, http: &Arc<Http>) -> AnyResult<()> {
        match self {
            TaskType::DateConditionalTask(task) => task.act(data, http).await,
            TaskType::Gulag(task) => task.act(data, http).await,
            TaskType::PeriodicTask(task) => task.act(data, http).await,
        }
    }
}
