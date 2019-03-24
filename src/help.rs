use super::*;

command!(Help(_context, message) {
    let _ = message.delete();
    println!("Start handling help command.");
    let start = Instant::now();
    if check_administrator(message.member()) {
        // Construct and send an embedded message
        let _ = message.channel_id.send_message(|m| m.embed(|e| e.title("Help")
            .colour(Colour::from_rgb(243, 44, 115))
            .description("Available commands and how to use them.")
            .field("=>gulag", "Send a user to the gulag.\nGeneral use: `=>gulag USER_ID DURATION\n\
                Example use: `~!gulag 303468267308318721 1w 2d`\nNotes on duration: duration \
                arguments are expected to be in a <number><unit specifier> format. Available \
                specifiers are `w`, `d`, `h`, and `m` for week, day, hour, and minute \
                respectively.", false)
            .field("=>current-gulags", "Lists the ongoing gulag sentences.", false)
            .field("=>remove-gulag-info", "Remove the persistent gulag info for a user.\nUsage: \
                `=>remove-gulag-info USER_ID`, e.g. `=>remove-gulag-info 164099453408509963`",
                false)
            .field("~!help", "Shows this message.", false)
            .footer(|f| f.text("Your friendly, neighbourhood gulag officer, Officer Velvet")
                .icon_url(EMBED_ICON_URL))))?;
        println!("    Success!");
        println!("    Elapsed: {:?}", start.elapsed());
    } else {
        let r = message.reply("This's bot's for administrator use only, silly.")?;
        println!("    Permissions error.");
        println!("    Elapsed: {:?}", start.elapsed());
        delete_message_after_delay(r, 10);
    }
});