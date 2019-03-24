use super::*;

// This file just contains some QoL stuff. Nothing important.

pub fn check_administrator(opt_member: Option<Member>) -> bool {
    if let Some(member) = opt_member {
        if let Ok(perms) = member.permissions() {
            perms.administrator()
        } else {
            false
        }
    } else {
        false
    }
}

pub fn delete_message_after_delay(message: Message, delay: u64) {
    thread::spawn(move || {
        sleep(Duration::from_secs(delay));
        let _ = message.delete();
    });
}