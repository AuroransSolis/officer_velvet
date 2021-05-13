use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Colour,
};
use std::time::Instant;

#[command]
pub async fn list_tasks(ctx: &Context, message: &Message) -> CommandResult {
    println!("LT | Start handling list tasks command.");
    let start = Instant::now();
    println!("LT | Grabbing read 'lock' on context data.");
    Ok(())
}
