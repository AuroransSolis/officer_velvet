#![allow(unused_imports)]

#[macro_use] extern crate serenity;
extern crate byteorder;

use serenity::{
    model::{
        id::{UserId, GuildId, RoleId},
        channel::{Message}, guild::{PartialGuild, Member, Role}
    },
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
mod remove_gulag_info;
use remove_gulag_info::RemoveGulagInfo;
use std::alloc::System;

pub const COUNTER_FILE: &str = "./activity_counter";
pub const GULAG_DIR: &str = "./gulags";
pub const EMBED_ICON_URL: &str = "https://cdn.discordapp.com/avatars/555257721587499038/\
    d1e248dc6720d3484c97bab2bf03e75f.png";
pub const GATHERING_PERIOD: u64 = 604800; // one week in seconds
pub const AURO_UID: UserId = UserId(246497842909151232);
pub const CRAK_UID: UserId = UserId(221345168463364098);
pub const BOT_UID: UserId = UserId(555257721587499038);
pub const AXOLOTL_ARMADA_GID: GuildId = GuildId(549382175703957504);

pub const WEEK_AS_SECS: u64 = 604800;
pub const DAY_AS_SECS: u64 = 86400;
pub const HOUR_AS_SECS: u64 = 3600;
pub const MIN_AS_SECS: u64 = 60;

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
    let token = env::var("VELVET")
        .expect("Expected a token in the environment");
    println!("Retrieved bot token.");
    // Create activity counter file if one doesn't exist
    if !Path::new(COUNTER_FILE).is_file() {
        let mut file = File::create(COUNTER_FILE).unwrap();
        let _ = file.write_u64::<LittleEndian>(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()).unwrap();
        let _ = file.write_u64::<LittleEndian>(0).unwrap();
    }
    // Create client with the handler that I've defined
    let mut client = Client::new(&token, Handler).expect("Err creating client");
    println!("Created client.");
    // Cache a PartialGuild. Requesting one of these can be expensive and cause ratelimiting issues,
    // so we'll just cache one at the start. Most of the things in it aren't useful, but I do need
    // the PartialGuild itself for adding and removing roles. Also cache the Prisoner role so that
    // we don't have to fetch it every time we want to use it.
    let partial_guild = PartialGuild::get(AXOLOTL_ARMADA_GID)
        .expect("Failed to get PartialGuild from GuildId(549382175703957504)");
    let gulag_role = partial_guild.role_by_name("Prisoner").expect("Failed to get gulag role.")
        .clone();
    let _ = client.data.lock().insert::<GulagRole>(gulag_role.clone());
    let _ = client.data.lock().insert::<CachedPartialGuild>(partial_guild);
    // Load the gulag sentences - see start_gulag_sentences(/* args */) and
    // load_gulag_sentences(/* args */) in gulag_handling.rs.
    start_gulag_sentences(gulag_role.id, load_gulag_sentences());
    // Configure the client
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("=>"))
        .cmd("gulag", Gulag)
        .cmd("current-gulags", CurrentGulags)
        .cmd("remove-gulag-info", RemoveGulagInfo)
        .cmd("help", Help));
    println!("Starting client.");// Start client
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}