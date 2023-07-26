pub mod date_conditional_task;
pub mod gulag;
pub mod message;
pub mod periodic_task;
pub mod task;

use crate::{
    cache_keys::TaskSenderKey,
    help::CREATE_TASK_HELP_MSG,
    misc::{insufficient_perms, is_administrator, ClapResult},
};
use anyhow::Result as AnyResult;
use clap::{error::ErrorKind, ColorChoice, Parser, Subcommand};
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
use std::{ffi::OsString, sync::Arc, time::Instant};
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
        matches!(self, TaskType::Gulag(_))
    }

    pub async fn act(
        &mut self,
        data: &Arc<RwLock<TypeMap>>,
        http: &impl AsRef<Http>,
    ) -> AnyResult<()> {
        match self {
            TaskType::DateConditionalTask(task) => task.act(data, http).await,
            TaskType::Gulag(task) => task.act(data, http.as_ref()).await,
            TaskType::PeriodicTask(task) => task.act(data, http).await,
        }
    }

    pub fn gulag_ref(&self) -> Option<&Gulag> {
        match self {
            TaskType::Gulag(gulag) => Some(gulag),
            _ => None,
        }
    }

    pub fn gulag_mut(&mut self) -> Option<&mut Gulag> {
        match self {
            TaskType::Gulag(gulag) => Some(gulag),
            _ => None,
        }
    }

    pub fn list_fmt(&self) -> String {
        match self {
            TaskType::DateConditionalTask(dct) => dct.list_fmt(),
            TaskType::Gulag(g) => g.list_fmt(),
            TaskType::PeriodicTask(pt) => pt.list_fmt(),
        }
    }
}

lazy_static! {
    static ref CTREGEX: Regex = Regex::new(r"=>create_task (.+)(\n```json\n([.\n]+)```)?").unwrap();
}

#[derive(Parser)]
#[command(color(ColorChoice::Never), no_binary_name(true))]
pub struct CreateTask {
    #[command(subcommand)]
    cttype: CreateTaskType,
}

#[derive(Clone, Debug, Subcommand)]
pub enum CreateTaskType {
    #[command(name = "date_conditional_task")]
    DateConditionalTask(DateConditionalTask),
    #[command(name = "periodic_task")]
    PeriodicTask(CreatePeriodicTask),
}

impl CreateTaskType {
    pub fn create(self) -> TaskType {
        match self {
            CreateTaskType::DateConditionalTask(dct) => TaskType::DateConditionalTask(dct),
            CreateTaskType::PeriodicTask(pt) => TaskType::PeriodicTask(pt.create()),
        }
    }
}

fn get_ctt_matches(msg: &str) -> Vec<Vec<&str>> {
    CTREGEX
        .captures_iter(msg)
        .map(|caps| {
            caps.iter()
                .skip(1)
                .filter_map(|cap| cap.map(|cap| cap.as_str()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

fn try_get_createtask<I>(iter: I) -> ClapResult<CreateTaskType>
where
    I: Iterator,
    <I as Iterator>::Item: Clone + Into<OsString>,
{
    CreateTask::try_parse_from(iter)
        .map_err(Into::into)
        .map(|ct| ct.cttype)
}

#[command]
pub async fn create_task(ctx: &Context, message: &Message) -> CommandResult {
    println!("CT | Begin handling create task command.");
    let start = Instant::now();
    if is_administrator(&ctx.http, ctx.data.read().await, message).await? {
        println!("CT | User has sufficient permissions. Trying to match subcommand.");
        let matches = get_ctt_matches(message.content.as_str());
        println!("CT | Regex matches: {matches:?}");
        if matches.is_empty() {
            println!("CT | User didn't provide all arguments, or failed to match format.");
            match message.content.to_lowercase().as_str() {
                "=>create_task -h" | "=>create_task --help" => {
                    let msg = format!(
                        "Error parsing command. Details:\n{}",
                        CREATE_TASK_HELP_MSG.as_str()
                    );
                    message
                        .channel_id
                        .send_message(&ctx.http, |message| message.content(&msg))
                        .await?;
                    return Ok(());
                }
                _ => {
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
            }
        } else if matches.len() == 1 {
            println!("CT | Valid user input.");
            let input = &matches[0];
            println!("CT | Parsing input: {input:?}");
            let mut subcommand = match try_get_createtask(input.iter()) {
                Ok(subcommand) => subcommand.create(),
                Err(err) if err.kind() == ErrorKind::DisplayHelp => {
                    println!("CT | User requested help.");
                    let msg = format!("```{err}```");
                    message.reply(&ctx.http, msg).await?;
                    println!("CT | Elapsed: {:?}", start.elapsed());
                    return Ok(());
                }
                Err(err) => {
                    println!("CT | Failed to parse user input. Sending error back.");
                    let msg = format!("Error parsing command. Details:\n```{err}```");
                    message.reply(&ctx.http, msg).await?;
                    println!("CT | Elapsed: {:?}", start.elapsed());
                    return Err(err.into());
                }
            };
            println!(
                "CT | PS | Successfully parsed task type: {}",
                subcommand.list_fmt()
            );
            // let task = match serde_json::from_str::<Task>(task) {
            //     Ok(task) => task,
            //     Err(err) => {
            //         let msg = format!(
            //             "Failed to parse task from input. Error details:\n```{}```",
            //             err
            //         );
            //         message.reply(&ctx.http, msg.as_str()).await?;
            //         return Ok(());
            //     }
            // };
            // println!("CT | PS | Successfully parsed task input.");
            // match &mut subcommand {
            //     TaskType::DateConditionalTask(DateConditionalTask {
            //         task: default_task,
            //         ..
            //     })
            //     | TaskType::PeriodicTask(PeriodicTask {
            //         task: default_task, ..
            //     }) => {
            //         *default_task = task;
            //     }
            //     TaskType::Gulag(_) => unreachable!(),
            // }
            // println!("CT | Assigned task to tasktype.");
            // let _ = &ctx
            //     .data
            //     .write()
            //     .await
            //     .get_mut::<TaskSenderKey>()
            //     .unwrap()
            //     .send(subcommand)?;
            // println!("CT | Sent task to executor.");
        } else {
            println!("CT | What the fuck: {matches:?}");
        }
    } else {
        println!("CT | User has insufficient permissions.");
        insufficient_perms(ctx, message).await?;
    }
    println!("CT | Elapsed: {:?}", start.elapsed());
    Ok(())
}
