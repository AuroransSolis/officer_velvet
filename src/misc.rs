use crate::cache_keys::ConfigKey;
use anyhow::Result;
use serenity::{http::CacheHttp, model::channel::Message, prelude::*};
use tokio::sync::RwLockReadGuard;

// This file just contains some QoL stuff. Nothing important.

pub async fn is_administrator(
    http: impl CacheHttp,
    context_data: RwLockReadGuard<'_, TypeMap>,
    message: &Message,
) -> Result<bool> {
    println!("    Getting user's roles.");
    let user_roles = message.member(http).await?.roles;
    Ok(context_data
        .get::<ConfigKey>()
        .unwrap()
        .elevated_roles
        .iter()
        .any(|(_, role_id)| user_roles.contains(role_id)))
}

pub async fn insufficient_perms(ctx: &Context, message: &Message) -> Result<()> {
    println!("    User has insufficient permissions. Notifying and returning.");
    let _ = message
        .reply(
            &ctx.http,
            "You'd best slide me over a bit of the good-good, comrade, or the officers will hear \
        about your attempt to usurp authority.",
        )
        .await?;
    Ok(())
}

/*pub fn delete_message_after_delay(message: Message, delay: u64) {
    thread::spawn(move || {
        sleep(Duration::from_secs(delay));
        let _ = message.delete();
    });
}*/
