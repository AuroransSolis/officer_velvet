use crate::{
    tasks::{message::MessageType, task::Task, TaskType},
    BotIdKey, TasksKey,
};
use serenity::{
    async_trait,
    framework::standard::{macros::hook, CommandError},
    model::{channel::Message, prelude::Ready},
    prelude::*,
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, message: Message) {
        if message.author.id != *context.data.read().await.get::<BotIdKey>().unwrap() {
            println!("HL | Begin handling sent message.");
            // Get a lock on the data by holding onto the result of `write`.
            let mut data = context.data.write().await;
            // Get the task list.
            let tasks = data.get_mut::<TasksKey>().unwrap();
            // Try and find a periodic task with the header "Activity report".
            let try_find_counter = tasks.iter_mut().find_map(|task_type| match task_type {
                TaskType::PeriodicTask(task) => match &mut task.task {
                    Task::SendMessage { message, .. } => match message {
                        MessageType::Embed(msg)
                            if msg.as_ref().description.as_deref() == Some("Activity report") =>
                        {
                            msg.as_mut().fields.as_mut().unwrap().get_mut(0)
                        }
                        _ => None,
                    },
                    Task::UpdateAppearance { .. } => None,
                },
                _ => None,
            });
            // If it exists, parse the number the content contains, increment the number, and
            // update the content.
            if let Some((_, counter, _)) = try_find_counter {
                let mut current_count = counter.parse::<usize>().unwrap();
                current_count += 1;
                counter.clear();
                counter.push_str(&format!("{}", current_count));
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("HD | Connected as user '{}'.", ready.user.name);
    }
}

#[hook]
pub async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: Result<(), CommandError>) {
    if let Err(why) = error {
        println!(
            "Command {:?} triggered by {}: {:?}",
            cmd_name,
            msg.author.tag(),
            why
        );
        let _ = msg.react(ctx, '\u{274C}').await;
    }
}
