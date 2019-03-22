use super::*;

pub const WEEK_AS_SECS: u64 = 604800;
pub const DAY_AS_SECS: u64 = 86400;
pub const HOUR_AS_SECS: u64 = 3600;
pub const MIN_AS_SECS: u64 = 60;

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
        println!("    Elapsed: {:?}", $instant.elapsed());
        delete_message_after_delay(r, $delay);
        return Ok(());
    }};
}

// ~!gulag 246497842909151232 1d 15h 30m 45s
command!(Gulag(context, message, args) {
    println!("Commence the gulagging.");
    let start = Instant::now();
    let _ = message.delete();
    if check_administrator(message.member()) {
        let gulag_role = if let Some(cache) = context.data.try_lock() {
            cache.get::<GulagRole>().id
        } else {
            reply_log_return!(start, 10, message,
                "Failed to get gulag role from cache. Please try again.");
        };
        let user_id = parse_arg_ok_or_return!(u64, start, 10, message,
            "Failed to parse first argument (user ID)");
        let user_id = UserId(user_id);
        let mut duration_arguments = Vec::new();
        while let Ok(arg) = args.single::<String>() {
            duration_arguments.push(arg);
        }
        let duration = duration_arguments.into_iter().map(|arg| {
            if arg.chars().filter(|c| !c.is_digit(10)).sum::<usize>() > 1 {
                reply_log_return!(start, 10, message,
                    format!("Invalid duration argument: {}", arg).as_str());
            }
            let arg_digits = arg.chars().take_while(|c| c.is_digit(10)).collect::<String>();
            let arg_val = match arg_digits.parse::<u64>() {
                Ok(val) => val,
                Err(_) => reply_log_return!(start, 10, message,
                    format!("Failed to parse number in argument: {}", arg).as_str())
            };
            match arg.chars().rev().next().unwrap() {
                'w' => arg_val * WEEK_AS_SECS,
                'd' => arg_val * DAY_AS_SECS,
                'h' => arg_val * HOUR_AS_SECS,
                'm' => arg_val * MIN_AS_SECS,
                 _ => {
                    let msg = format!("Invalid time unit specifier in argument: {}", arg);
                    let r = message.reply(msg.as_str())?;
                    println!("    {}", msg);
                    println!("    Elapsed: {:?}", start.elapsed());
                    delete_message_after_delay(r, 10);
                    return Ok(());
                }
            }
        }).sum::<u64>();
        if let Some(gulag_entry) = write_gulag_file(duration, user_id, &message, &context) {
            if start_new_gulag_sentence(&context, &message, gulag_entry) {
                let r = message.reply("Successfully gulagged user!")?;
                println!("    Success!");
                println!("    Elapsed: {:?}", start.elapsed());
                delete_message_after_delay(r, 10);
            } else {
                let file_string = format!("{}/{}.gulag", GULAG_DIR, user_id.0);
                if let Err(_) = remove_file(file_string.as_str()) {
                    match AURO_UID.to_user() {
                        Ok(user) => drop(user.direct_message(|m| m.content(file_string.as_str()))),
                        Err(_) => println("    REMOVE FILE BY HAND: {}", file_string)
                    }
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