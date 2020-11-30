use crate::misc::escape_formatting;
use rand::{seq::SliceRandom, thread_rng};
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use std::time::Instant;

#[command]
pub async fn anagram(ctx: &Context, message: &Message) -> CommandResult {
    println!("Received anagram command.");
    let start = Instant::now();
    let unscrambled = message.content.trim_start_matches("=>anagram").trim();
    println!("    Trimmed '=>anagram' from message.");
    if unscrambled.len() == 0 {
        let _ = message
            .reply(
                &ctx.http,
                "The fuck am I supposed to be scrambling, peasant?",
            )
            .await?;
    } else if unscrambled.len() == 1 {
        let _ = message
            .reply(
                &ctx.http,
                "I am in awe of your incompetence if you need help scrambling that.",
            )
            .await?;
    } else {
        let first = unscrambled.chars().next().unwrap();
        if !unscrambled.chars().any(|c| c != first) {
            let _ = message
                .reply(
                    &ctx.http,
                    "I am in awe of your incompetence if you need help scrambling that.",
                )
                .await?;
        } else {
            let mut scrambled = unscrambled.chars().collect::<Vec<char>>();
            println!("    Collected trimmed message into Vec<char>.");
            scrambled.shuffle(&mut thread_rng());
            println!("    Shuffled message.");
            let scrambled = escape_formatting(scrambled.into_iter().collect::<String>());
            println!("    Collected shuffled characters into string.");
            let msg = format!(
                "Hey guys, did you know that \"{}\" is an anagram of \"{}\"?",
                scrambled, unscrambled
            );
            println!("    Formatted message.");
            if msg.len() < 2000 {
                let _ = message
                    .channel_id
                    .send_message(&ctx.http, |m| m.content(&msg))
                    .await?;
            } else {
                let reply = format!(
                    "That's {} characters too many. This incident will be recorded on your record.",
                    msg.len() - 2000
                );
                let _ = message.reply(&ctx.http, &reply).await?;
            }
        }
    }
    println!("    Sent reply.");
    println!("    Elapsed: {:?}", start.elapsed());
    Ok(())
}
