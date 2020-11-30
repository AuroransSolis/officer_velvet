pub mod date_conditional_task;
pub mod gulag;
pub mod message;
pub mod periodic_task;
pub mod task;

use crate::{
    cache_keys::TaskSenderKey,
    misc::{insufficient_perms, is_administrator},
};
use anyhow::Result as AnyResult;
use date_conditional_task::DateConditionalTask;
use gulag::Gulag;
use lazy_static::lazy_static;
use periodic_task::{CreatePeriodicTask, PeriodicTask};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    http::client::Http,
    model::channel::Message,
    prelude::{RwLock, TypeMap},
};
use std::{sync::Arc, time::Instant};
use structopt::{clap::AppSettings, StructOpt};
use task::Task;

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

lazy_static! {
    static ref CTREGEX: Regex = Regex::new(r"=>create_task (.+)\n```json\n([\w\W]+)```").unwrap();
}

#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "Create Task",
    author = "Aurorans Solis",
    settings(&[AppSettings::ColorNever, AppSettings::NoBinaryName]),
)]
pub enum CreateTaskType {
    #[structopt(name = "date_conditional_task")]
    DateConditionalTask(DateConditionalTask),
    #[structopt(name = "periodic_task")]
    PeriodicTask(CreatePeriodicTask),
}

impl CreateTaskType {
    pub fn create(self) -> AnyResult<TaskType> {
        Ok(match self {
            CreateTaskType::DateConditionalTask(dct) => TaskType::DateConditionalTask(dct),
            CreateTaskType::PeriodicTask(pt) => TaskType::PeriodicTask(pt.create()?),
        })
    }
}

fn get_ctt_matches(msg: &str) -> Vec<(&str, &str)> {
    CTREGEX
        .captures_iter(msg)
        .map(|caps| (caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str()))
        .collect::<Vec<_>>()
}

fn try_get_createtask(s: &str) -> AnyResult<CreateTaskType> {
    CreateTaskType::from_iter_safe(s.trim().split_whitespace()).map_err(|e| e.into())
}

#[command]
pub async fn create_task(ctx: &Context, message: &Message) -> CommandResult {
    println!("CT | Begin handling create task command.");
    let start = Instant::now();
    if is_administrator(&ctx.http, ctx.data.read().await, message).await? {
        println!("CT | User has sufficient permissions. Trying to match subcommand.");
        match &get_ctt_matches(message.content.as_str())[..] {
            &[] => {
                println!("CT | User didn't provide all arguments, or failed to match format.");
                message
                    .reply(
                        &ctx.http,
                        "Aye, I'll be sure to do nothing.\n\
                        \n\
                        Sarcasm aside, I didn't find any tasks in that message. \
                        Double-check your usage.",
                    )
                    .await?;
            }
            &[(subcommand, task)] => {
                println!("CT | Valid user input.");
                let mut subcommand = try_get_createtask(subcommand)?.create()?;
                println!("CT | PS | Successfully parsed task type input.");
                let task = match serde_json::from_str::<Task>(task) {
                    Ok(task) => task,
                    Err(err) => {
                        let msg = format!(
                            "Failed to parse task from input. Error details:\n```{}```",
                            err
                        );
                        message.reply(&ctx.http, msg.as_str()).await?;
                        return Ok(());
                    }
                };
                println!("CT | PS | Successfully parsed task input.");
                match &mut subcommand {
                    TaskType::DateConditionalTask(DateConditionalTask {
                        task: default_task,
                        ..
                    })
                    | TaskType::PeriodicTask(PeriodicTask {
                        task: default_task, ..
                    }) => {
                        *default_task = task;
                    }
                    _ => unreachable!(),
                }
                println!("CT | Assigned task to tasktype.");
                &ctx.data
                    .write()
                    .await
                    .get_mut::<TaskSenderKey>()
                    .unwrap()
                    .send(subcommand)?;
                println!("CT | Sent task to executor.");
            }
            _ => {
                println!("CT | User provided too many tasks.");
                message
                    .reply(
                        &ctx.http,
                        "Woah there, you can only throw so much at me. \
                            I only accept new tasks one at a time.",
                    )
                    .await?;
            }
        }
    } else {
        println!("CT | User has insufficient permissions.");
        insufficient_perms(ctx, message).await?;
    }
    println!("CT | Elapsed: {:?}", start.elapsed());
    Ok(())
}
