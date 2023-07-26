use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use std::time::Instant;

use crate::cache_keys::TasksKey;

#[command]
pub async fn list_tasks(ctx: &Context, message: &Message) -> CommandResult {
    println!("LT | Start handling list tasks command.");
    let start = Instant::now();
    println!("LT | Grabbing read 'lock' on context data.");
    let context_data = ctx.data.read().await;
    println!("LT | Grabbing tasks from context data.");
    let tasks = context_data.get::<TasksKey>().unwrap().as_slice();
    println!("LT | Formatting message contents.");
    let msg = if tasks.is_empty() {
        "No tasks currently!".into()
    } else {
        let mut msg = "Current task list:\n```".to_string();
        tasks.iter().enumerate().for_each(|(ind, task)| {
            msg.push('\n');
            msg.push_str(&format!("{ind}: "));
            let add = task.list_fmt();
            msg.push_str(&add);
        });
        msg.push_str("\n```");
        msg
    };
    drop(context_data);
    println!("LT | Sending message.");
    message
        .channel_id
        .send_message(&ctx.http, |m| m.content(msg))
        .await?;
    println!("LT | Elapsed: {:?}", start.elapsed());
    Ok(())
}
