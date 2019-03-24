use super::*;

// Two convenience macros. You can't stop me.
macro_rules! parse_arg_ok_or_return {
    ($args:ident, $parse_type:ty, $instant:ident, $delay:literal, $message:ident, $reply:expr) => {{
        match $args.single::<$parse_type>() {
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

command!(Gulag(context, message, args) {
    println!("Commence the gulagging.");
    let start = Instant::now();
    let _ = message.delete();
    if check_administrator(message.member()) {
        // Get input user ID
        let user_id = parse_arg_ok_or_return!(args, u64, start, 10, message,
            "Failed to parse first argument (user ID)");
        let user_id = UserId(user_id);
        // Collect the duration arguments into a vector
        let mut duration_arguments = Vec::new();
        while let Ok(arg) = args.single::<String>() {
            duration_arguments.push(arg);
        }
        // Sum variable for the total duration
        let mut duration = 0;
        // For each duration argument...
        for duration_arg in duration_arguments {
            // Count the number of characters that are not digits...
            if duration_arg.chars().filter(|&c| (c as u8) < ('0' as u8) || (c as u8) > ('9' as u8))
                .count() > 1 {
                // ...and let the user know that the duration argument is invalid if there's more than
                // one non-digit character in the argument.
                reply_log_return!(start, 10, message,
                    format!("Invalid duration argument: {}", duration_arg).as_str());
            }
            // Collect all the digits in the argument
            let arg_digits = duration_arg.chars()
                .take_while(|&c| (c as u8) < ('0' as u8) || (c as u8) > ('9' as u8))
                .collect::<String>();
            let arg_val = match arg_digits.parse::<u64>() {
                Ok(val) => val,
                Err(_) => reply_log_return!(start, 10, message,
                    format!("Failed to parse number in argument: {}", duration_arg).as_str())
            };
            // Multiply the digits by the appropriate amount based on the last character and add
            // that to the total
            duration += match duration_arg.chars().rev().next().unwrap() {
                'w' => arg_val * WEEK_AS_SECS,
                'd' => arg_val * DAY_AS_SECS,
                'h' => arg_val * HOUR_AS_SECS,
                'm' => arg_val * MIN_AS_SECS,
                's' => arg_val,
                 _ => {
                    let msg = format!("Invalid time unit specifier in argument: {}", duration_arg);
                    let r = message.reply(msg.as_str())?;
                    println!("    {}", msg);
                    println!("    Elapsed: {:?}", start.elapsed());
                    delete_message_after_delay(r, 10);
                    return Ok(());
                }
            };
        }
        // Attempt to write the gulag file
        if let Some(gulag_entry) = write_gulag_file(duration, user_id, &message) {
            // Attempt to start a new gulag sentence
            if start_new_gulag_sentence(&context, &message, gulag_entry) {
                let r = message.reply("Successfully gulagged user!")?;
                println!("    Success!");
                delete_message_after_delay(r, 10);
            } else {
                // If writing the file but gulagging the user fails, try to delete the gulag file.
                let file_string = format!("{}/{}.gulag", GULAG_DIR, user_id.0);
                if let Err(_) = remove_file(file_string.as_str()) {
                    match AURO_UID.to_user() {
                        Ok(user) => drop(user.direct_message(|m| m.content(file_string.as_str()))),
                        Err(_) => println!("    REMOVE FILE BY HAND: {}", file_string)
                    }
                }
            }
        } else {
            let r = message.reply("Failed to write persistent gulag info. Please try again.")?;
            println!("    Failed to write persistent info to file.");
            println!("    Elapsed: {:?}", start.elapsed());
            delete_message_after_delay(r, 10);
        }
    } else {
        let r = message.reply("You have to be an administrator to do that.")?;
        println!("    Permissions error.");
        delete_message_after_delay(r, 10);
    }
    println!("    Elapsed: {:?}", start.elapsed());
});