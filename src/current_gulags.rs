use super::*;

const SING_UNIT_STRINGS: [&str; 5] = ["week", "day", "hour", "minute", "second"];
const MULT_UNIT_STRINGS: [&str; 5] = ["weeks", "days", "hours", "minutes", "seconds"];

command!(CurrentGulags(_context, message) {
    let _ = message.delete();
    println!("Start handling current-gulags command.");
    let start = Instant::now();
    if check_administrator(message.member()) {
        // Create a collection to store an informative blurb about each directory entry
        let mut entries = Vec::new();
        let dir_iter = read_dir(GULAG_DIR).expect("Failed to read contents of gulags directory.");
        for item in dir_iter {
            if let Ok(entry) = item {
                // Get the user ID specified by the filename
                let mut file_name = entry.file_name().into_string().unwrap();
                for _ in 0..6 {
                    let _ = file_name.pop();
                }
                println!("    Found file for user ID: {:?}", file_name);
                // Open the file and read the offset. Then convert that into time left in the
                // gulag sentence.
                let mut file = File::open(entry.path().as_path()).unwrap();
                let offset = file.read_u64::<LittleEndian>().unwrap();
                let mut left_to_offset = (SystemTime::UNIX_EPOCH + Duration::from_secs(offset))
                    .duration_since(SystemTime::now()).unwrap().as_secs();
                let weeks = left_to_offset / WEEK_AS_SECS;
                left_to_offset %= WEEK_AS_SECS;
                let days = left_to_offset / DAY_AS_SECS;
                left_to_offset %= DAY_AS_SECS;
                let hours = left_to_offset / HOUR_AS_SECS;
                left_to_offset %= HOUR_AS_SECS;
                let minutes = left_to_offset / MIN_AS_SECS;
                left_to_offset %= MIN_AS_SECS;
                let seconds = left_to_offset;
                let time_unit_amts = [weeks, days, hours, minutes, seconds];
                // Create a string in the format: "USER_ID | TIME_LEFT
                // TIME_LEFT will contain info on the number of week(s), day(s), hour(s), minute(s),
                // and second(s) left in the sentence.
                let mut entry_string = format!("{} |", file_name);
                for (i, &unit) in time_unit_amts.iter().enumerate() {
                    if unit == 1 {
                        entry_string = format!("{}{} {}, ", entry_string, unit,
                            SING_UNIT_STRINGS[i]);
                    } else if unit > 0 {
                        entry_string = format!("{}{} {}, ", entry_string, unit,
                            MULT_UNIT_STRINGS[i]);
                    }
                }
                entries.push(entry_string);
            }
        }
        // Create a String that will be a collection of all of the entries
        let mut list = String::new();
        if entries.len() == 0 {
            list += "No users are currently gulagged.";
        } else if entries.len() == 1 {
            list = entries.remove(0);
        } else {
            list = entries.remove(0);
            for entry in entries.into_iter().skip(1) {
                list = format!("{}\n{}", list, entry);
            }
        }
        // Create and send an embedded message with information on the current gulag sentences.
        let _ = message.channel_id.send_message(|m| m.embed(|e| e.title("Current gulag sentences:")
            .colour(Colour::from_rgb(243, 44, 115))
            .field("User ID | Time left", list.as_str(), false)
            .footer(|f| f.text("Your friendly, neighbourhood gulag officer, Officer Velvet")
                .icon_url("https://cdn.discordapp.com/avatars/246497842909151232/\
                    6a452f7523d2e37a35bfa70863bfa679.png"))))?;
    } else {
        let r = message.reply("You must be an administrator to do that.")?;
        println!("    Permissions error.");
        println!("    Elapsed: {:?}", start.elapsed());
        delete_message_after_delay(r, 10);
    }
});