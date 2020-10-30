use crate::{
    cache_keys::{ConfigKey, TasksKey},
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
    println!("Start handling current-gulags command.");
    let start = Instant::now();
    println!("    Getting user's roles.");
    let user_roles = message.member(&ctx.http).await?.roles;
    println!("    Grabbing read 'lock' on context data.");
    let context_data = ctx.data.read().await;
    println!("    Checking permissions.");
    if context_data
        .get::<ConfigKey>()
        .unwrap()
        .elevated_roles
        .iter()
        .any(|(_, role_id)| user_roles.contains(role_id))
    {
        println!("    User has sufficient permissions.");
        let gulags = context_data
            .get::<TasksKey>()
            .unwrap()
            .iter()
            .filter_map(|task_type| match task_type {
                TaskType::Gulag(gulag) => Some(gulag),
                _ => None,
            })
            .collect::<Vec<_>>();
        println!("    Got list of active gulag sentences.");
        let msg = if gulags.is_empty() {
            "Nobody is currently gulagged.".into()
        } else {
            let mut msg = String::new();
            for gulag in gulags {
                msg.push_str(&gulag.to_string());
                msg.push('\n');
            }
            msg.trim_end().to_string()
        };
        println!("    Formatted gulag sentences.");
        let icon_url = context_data.get::<ConfigKey>().unwrap().icon_url.as_str();
        println!("    Retrieved icon URL.");
        let _ = message
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("Prisoner List")
                        .colour(Colour::from_rgb(243, 44, 115))
                        .field("Report from the tundra", msg.as_str(), false)
                        .footer(|f| {
                            f.text("Your friendly, neighbourhood gulag officer, Officer Velvet")
                                .icon_url(icon_url)
                        })
                })
            })
            .await?;
        println!("    Sent gulags list.");
    } else {
        println!("    User has insufficient permissions. Notifying and returning.");
        let _ = message.reply(
            &ctx.http,
            "You'd best slide me over a bit of the good-good, comrade, or the officers will hear \
            about your attempt to usurp authority."
        ).await?;
    }
    println!("    Elapsed: {:?}", start.elapsed());
    Ok(())
}
