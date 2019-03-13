#[macro_use] extern crate serenity;
extern crate byteorder;

use serenity::{
    model::{id::{ChannelId, MessageId, UserId, GuildId, RoleId}, channel::{Message},
        guild::{PartialGuild, Member}},
    prelude::*,
    framework::standard::StandardFramework
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::fs::{File, read_dir};
use std::path::Path;
use std::time::{Instant, Duration};
use std::thread::{self, sleep};
use std::env;

mod handler;
use handler::Handler;
mod gulag;
use gulag::Gulag;

pub const COUNTER_FILE: &str = "./activity_counter";
pub const GULAG_DIR: &str = "./gulags";
pub const GATHERING_PERIOD: u64 = 604800; // one week in seconds

pub struct CachedPartialGuild;

impl TypeMapKey for CachedPartialGuild {
    type Value = PartialGuild;
}

pub struct GulagEntry {
    member_name: String,
    user_id: UserId,
    previous_roles: Vec<RoleId>,
    gulag_sentence: u64
}

fn main() {
    let token = env::var("TESTING")
        .expect("Expected a token in the environment");
    println!("Retrieved bot token.");
    let mut client = Client::new(&token, Handler).expect("Err creating client");
    println!("Created client.");
    let _ = client.data.lock().insert::<CachedPartialGuild>(
        PartialGuild::get(GuildId(549382175703957504))
            .expect("Failed to get PartialGuild from GuildId(549382175703957504)"));
    let partialguild = PartialGuild::get(GuildId(549382175703957504))
        .expect("Failed to get PartialGuild from GuildId(549382175703957504)");
    let foo = partialguild.member()
}

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

fn load_gulag_sentences() -> Vec<GulagEntry> {

}

pub fn write_gulag_file(member: Member) {
    let path= format!("{}/{}.gulag", GULAG_DIR, member.user_id());
    let name = member.display_name().into_string();

    let previous_roles = member.roles;
    let file = File::create(path.as_str()).expect("Failed to create new gulag file.");
}