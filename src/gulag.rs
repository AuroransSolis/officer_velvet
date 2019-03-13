use super::*;

const WEEK_AS_SECS: u64 = 604800;
const DAY_AS_SECS: u64 = 86400;
const HOUR_AS_SECS: u64 = 3600;
const MIN_AS_SECS: u64 = 60;

command!(Gulag(context, message, args) {
    println!("Commence the gulagging.");
    let start = Instant::now();
    let _ = message.delete();
    if check_administrator(message.member()) {
        let
    } else {
        let r = message.reply("You have to be an administrator to do that.")?;
        println!("    Permissions error.");
        delete_message_after_delay(r, 10);
    }
    println!("    Elapsed: {:?}", start.elapsed());
});