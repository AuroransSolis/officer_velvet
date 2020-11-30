use crate::{
    cache_keys::TasksKey,
    misc::{insufficient_perms, is_administrator},
    tasks::TaskType,
};
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Colour,
};
use std::time::Instant;

#[command]
#[aliases("current-gulags")]
pub async fn current_gulags(ctx: &Context, message: &Message) -> CommandResult {
    println!("CG | Start handling current-gulags command.");
    let start = Instant::now();
    println!("CG | Grabbing read 'lock' on context data.");
    let context_data = ctx.data.read().await;
    println!("CG | Checking permissions.");
    if is_administrator(&ctx.http, context_data, message).await? {
        println!("CG | User has sufficient permissions.");
        let mut msg = String::new();
        ctx.data
            .read()
            .await
            .get::<TasksKey>()
            .unwrap()
            .iter()
            .filter_map(|task_type| match task_type {
                TaskType::Gulag(gulag) => Some(gulag),
                _ => None,
            })
            .for_each(|gulag| {
                msg.push_str(&gulag.to_string());
                msg.push('\n');
            });
        let msg = if msg.is_empty() {
            "Nobody is currently gulagged."
        } else {
            msg.trim()
        };
        println!("CG | Formatted gulag sentences.");
        let icon_url = ctx.http.get_current_user().await?.avatar_url().unwrap();
        println!("CG | Retrieved icon URL.");
        let _ = message
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("Prisoner List")
                        .colour(Colour::from_rgb(243, 44, 115))
                        .field("Report from the tundra", msg, false)
                        .footer(|f| {
                            f.text("Your friendly, neighbourhood gulag officer, Officer Velvet")
                                .icon_url(icon_url)
                        })
                })
            })
            .await?;
        println!("CG | Sent gulags list.");
    } else {
        insufficient_perms(ctx, message).await?;
    }
    println!("CG | Elapsed: {:?}", start.elapsed());
    Ok(())
}
