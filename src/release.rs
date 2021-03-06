use crate::{
    cache_keys::{ConfigKey, TasksKey},
    misc::{insufficient_perms, is_administrator, update_task_list},
};
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use std::time::Instant;

#[command]
pub async fn release(ctx: &Context, message: &Message) -> CommandResult {
    println!("RL | Start handling release command.");
    let start = Instant::now();
    println!("RL | Getting lock on context data.");
    let context_data = ctx.data.read().await;
    println!("RL | Checking permissions.");
    if is_administrator(&ctx.http, context_data, message).await? {
        println!("RL | User has sufficient permissions.");
        println!("RL | Getting user ID to un-gulag.");
        let user_id = message
            .content
            .trim_start_matches("=>release ")
            .parse::<UserId>()?;
        println!("RL | Getting write lock on context data");
        let context_data = ctx.data.read().await;
        println!("RL | Getting tasks list.");
        let tasks = context_data.get::<TasksKey>().unwrap();
        println!("RL | Tasks:\n{:?}", tasks);
        println!("RL | Checking for gulag entry for user ID {}", user_id);
        if let Some(ind) = tasks.iter().position(|task| {
            task.gulag_ref()
                .map(|gulag| gulag.user.1 == user_id)
                .unwrap_or(false)
        }) {
            println!("RL | Found gulag entry.");
            tasks[ind]
                .gulag_ref()
                .unwrap()
                .act(&ctx.data, &ctx.http)
                .await?;
            println!("RL | Removing gulag entry from task list.");
            drop(tasks);
            drop(context_data);
            let mut context_data = ctx.data.write().await;
            context_data.get_mut::<TasksKey>().unwrap().remove(ind);
            println!("RL | Writing changes out to task list file.");
            update_task_list(
                context_data.get::<ConfigKey>().unwrap().tasks_file.as_str(),
                context_data.get::<TasksKey>().unwrap(),
            )
            .await?;
        } else {
            println!("RL | No gulag entry for provided user ID exists.");
            message
                .reply(
                    &ctx.http,
                    "That person isn't gulagged currently. Wanna change that?",
                )
                .await?;
        }
    } else {
        insufficient_perms(ctx, message).await?;
    }
    println!("RL | Elapsed: {:?}", start.elapsed());
    Ok(())
}
