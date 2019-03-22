#[macro_use] extern crate serenity;
extern crate byteorder;

use serenity::{
    model::{id::{ChannelId, MessageId, UserId, GuildId, RoleId}, channel::{Message},
        guild::{PartialGuild, Member, Role}, user::User},
    prelude::*,
    framework::standard::StandardFramework,
    utils::Colour
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::fs::{File, read_dir, remove_file};
use std::path::{Path, PathBuf};
use std::time::{Instant, Duration, SystemTime};
use std::thread::{self, sleep};
use std::env;

mod handler;
use handler::Handler;
mod gulag;
use gulag::Gulag;
mod current_gulags;
use current_gulags::CurrentGulags;
mod help;
use help::Help;
mod gulag_handling;
use gulag_handling::*;
mod misc;
use misc::*;

pub const COUNTER_FILE: &str = "./activity_counter";
pub const GULAG_DIR: &str = "./gulags";
pub const GATHERING_PERIOD: u64 = 604800; // one week in seconds
pub const AURO_UID: UserId = UserId(246497842909151232);

pub struct CachedPartialGuild;

impl TypeMapKey for CachedPartialGuild {
    type Value = PartialGuild;
}

pub struct GulagRole;

impl TypeMapKey for GulagRole {
    type Value = Role;
}

pub struct GulagEntry {
    file_path: PathBuf,
    user_id: UserId,
    previous_roles: Vec<RoleId>,
    gulag_sentence: u64
}

fn main() {
    // Pull bot token from environment (using env. var. so that I don't have to publish the code
    // with an API token in it).
    let token = env::var("TESTING")
        .expect("Expected a token in the environment");
    println!("Retrieved bot token.");
    // Create client with the handler that I've defined
    let mut client = Client::new(&token, Handler).expect("Err creating client");
    println!("Created client.");
    // Cache a PartialGuild. Requesting one of these can be expensive and cause ratelimiting issues,
    // so we'll just cache one at the start. Most of the things in it aren't useful, but I do need
    // the PartialGuild itself for adding and removing roles.
    let partial_guild = PartialGuild::get(GuildId(549382175703957504))
        .expect("Failed to get PartialGuild from GuildId(549382175703957504)");
    let _ = client.data.lock().insert::<GulagRole>(partial_guild.role_by_name("Prisoner")
        .expect("Failed to get gulag role.").clone());
    let _ = client.data.lock().insert::<CachedPartialGuild>(partial_guild);
    let gulags = load_gulag_sentences();
    start_gulag_sentences(&client.data, gulags);

}