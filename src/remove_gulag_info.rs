use super::*;

command!(RemoveGulagInfo(_context, message, args) {
    let _ = message.delete();
    println!("Start handling removal of persistent gulag data.");
    let start = Instant::now();
    if !check_administrator(message.member()) {
        let r = message.reply("You must be an administrator to use that command.")?;
        println!("    Permissions lacking. Done.");
        println!("    Elapsed: {:?}", start.elapsed());
        delete_message_after_delay(r, 5);
    } else {
        // Ensure that the user input a valid user ID
        let user_id = match args.single::<u64>() {
            Ok(name) => name,
            Err(_) => {
                let r = message.reply("Invalid user ID.")?;
                println!("    Invalid user ID argument.");
                println!("    Elapsed: {:?}", start.elapsed());
                delete_message_after_delay(r, 10);
                let _ = message.delete();
                return Ok(());
            }
        };
        let user_id = format!("{}.gulag", user_id);
        // Search for a file that fits the user ID specified by the user
        let dir_iter = read_dir(GULAG_DIR).expect("Unable to read contents of '/gulags'");
        let mut path = None;
        for item in dir_iter {
            match item {
                Ok(dir_entry) => {
                    println!("Checking name: {}", dir_entry.file_name().into_string().unwrap());
                    if dir_entry.file_name().into_string().unwrap() == user_id {
                        path = Some(dir_entry.path().to_str().unwrap().to_string());
                        break;
                    }
                }
                _ => {}
            }
        }
        // If a file exists, try to delete the file. If there was none, inform the user.
        if let Some(p) = path {
            if remove_file(p.clone()).is_err() {
                let r = message.reply("Failed to delete persistent gulag info. Please contact \
                    Auro to ensure the user isn't re-gulagged when the bot restarts.")?;
                println!("DELETE FILE MANUALLY ({:?})", p);
                println!("    Elapsed: {:?}", start.elapsed());
                delete_message_after_delay(r, 5);
                return Ok(());
            }
            let r = message.reply("Successfully deleted persistent gulag data.")?;
            println!("    Successfully deleted file.");
            println!("    Elapsed: {:?}", start.elapsed());
            delete_message_after_delay(r, 5);
            return Ok(());
        } else {
            let r = message.reply("Found no gulag sentences under the specified user ID.")?;
            println!("    Query turned up no results.");
            println!("    Elapsed: {:?}", start.elapsed());
            delete_message_after_delay(r, 10);
            return Ok(());
        }
    }
});