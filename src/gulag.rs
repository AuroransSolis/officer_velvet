use super::*;

const WEEK_AS_SECS: u64 = 604800;
const DAY_AS_SECS: u64 = 86400;
const HOUR_AS_SECS: u64 = 3600;
const MIN_AS_SECS: u64 = 60;

macro_rules! parse_arg_ok_or_return {
    ($parse_type:ty, $instant:ident, $delay:literal, $message:ident, $reply:expr) => {{
        match args.single::<$parse_type>() {
            Ok(arg) => arg,
            Err(_) => {
                let r = $message.reply($reply)?;
                println!("    {}", $reply);
                println!("    Elapsed: {:?}", $instant.elapsed());
                delete_message_after_delay(r, $delay);
                return Ok(());
            }
        }
    }};
}

macro_rules! reply_log_return {
    ($instant:ident, $delay:literal, $message:ident, $reply:expr) => {{
        let r = $message.reply($reply)?;
        println!("    {}", $reply);
        delete_message_after_delay(r, $delay);
        return Ok(());
    }};
}

command!(Gulag(context, message, args) {
    println!("Commence the gulagging.");
    let start = Instant::now();
    let _ = message.delete();
    if check_administrator(message.member()) {
        let user_id = parse_arg_ok_or_return!(u64, start, 10, message,
            "Failed to parse first argument (user ID)");
        let user_id = UserId(user_id);
        let duration = parse_arg_ok_or_return!(u64, start, 10, message,
            "Failed to parse second argument (gulag duration).");
        let duration = Duration::from_secs(duration);
        let member = if let Some(mutexguarded_data) = context.data.try_lock() {
            if let Some(partial_guild) = mutexguarded_data.get::<CachedPartialGuild>() {
                if let Ok(m) = partial_guild.member(user_id) {
                    m
                } else {
                    reply_log_return!(start, 10, message,
                        "Could not find Member for specified user ID.");
                }
            }
        }
    } else {
        let r = message.reply("You have to be an administrator to do that.")?;
        println!("    Permissions error.");
        delete_message_after_delay(r, 10);
    }
    println!("    Elapsed: {:?}", start.elapsed());
});