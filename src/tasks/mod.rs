pub mod date_conditional_task;
pub mod gulag;
pub mod periodic_task;

use date_conditional_task::DateConditionalTask;
use gulag::Gulag;
use periodic_task::PeriodicTask;
use serde::{Deserialize, Serialize};

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
}
