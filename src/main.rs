use anyhow::Result as AnyResult;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crossbeam_channel::{unbounded, Receiver, Sender};
use serenity::{
    framework::{StandardFramework, standard::macros::group},
    model::{
        guild::{PartialGuild, Role},
        id::UserId,
    },
    prelude::*,
};
/*use serenity::{
    framework::standard::StandardFramework,
    model::{
        channel::Message,
        guild::{Member, PartialGuild, Role},
        id::{ChannelId, GuildId, RoleId, UserId},
    },
    prelude::*,
    utils::Colour,
};*/
use serde::{Deserialize, Serialize};
use serde_json::error::{Category as JsonErrorCategory, Error as JsonError};
use std::fs::{self, read_dir, remove_file, File};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Write};
use std::path::{Path, PathBuf};
use std::thread::{self, sleep};
use std::time::{Duration, Instant, SystemTime};
use structopt::StructOpt;

mod anagram;
mod args;
mod cache_keys;
mod config;
mod handler;
mod tasks;

use anagram::*;
use cache_keys::*;
use config::Config;
use handler::{after, Handler};
use tasks::TaskType;

/*mod gulag;
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
use anagram::Anagram;
mod reginald;
use reginald::reginald_visits;
mod kirb_day;
use kirb_day::kirb_day_task;
mod source;
use source::Source;

pub const COUNTER_FILE: &str = "./activity_counter";
pub const GULAG_DIR: &str = "./gulags";
pub const EMBED_ICON_URL: &str = "https://cdn.discordapp.com/avatars/555257721587499038/\
    d1e248dc6720d3484c97bab2bf03e75f.png";
pub const GATHERING_PERIOD: u64 = 604800; // one week in seconds
pub const CRAK_UID: UserId = UserId(221345168463364098);
pub const BOT_UID: UserId = UserId(555257721587499038);
pub const SHIT_CHANNEL: ChannelId = ChannelId(549383666246090773);
pub const ANNOUNCEMENTS_CHANNEL: ChannelId = ChannelId(549385011107987477);
pub const AXOLOTL_ARMADA_GID: GuildId = GuildId(549382175703957504);

pub const WEEK_AS_SECS: u64 = 604800;
pub const DAY_AS_SECS: u64 = 86400;
pub const HOUR_AS_SECS: u64 = 3600;
pub const MIN_AS_SECS: u64 = 60;*/

#[group]
#[commands(anagram)]
struct GeneralCommands;

#[tokio::main]
async fn main() -> AnyResult<()> {
    let args::Args { config_file_path } = args::Args::from_args();
    let config_contents = match fs::read_to_string(&config_file_path) {
        Ok(contents) => Ok(contents),
        Err(error) => {
            if error.kind() == IoErrorKind::NotFound {
                println!(
                    "Config file not found. Attempting to create new default config file at '{}'",
                    config_file_path
                );
                let mut new_config_file = File::create(&config_file_path)?;
                let default_contents = serde_json::to_string(&Config::default()).unwrap();
                new_config_file.write_all(default_contents.as_bytes())?;
                println!("Created new config file and wrote defaults.");
            }
            Err(error)
        }
    }?;
    println!("Read config file contents.");
    let config = serde_json::from_str::<Config>(&config_contents)?;
    println!("Parsed config from config file contents.");
    let tasks = match fs::read_to_string(&config.tasks_file) {
        Ok(contents) if contents.len() == 0 => Ok(Vec::new()),
        Ok(contents) => Ok(serde_json::from_str::<Vec<TaskType>>(&contents)?),
        Err(error) => match error.kind() {
            IoErrorKind::NotFound => {
                println!(
                    "Tasks file not found. Attempting to create new tasks file at '{}'",
                    config.tasks_file
                );
                let _ = File::create(&config.tasks_file)?;
                println!("Created new blank tasks file.");
                Ok(Vec::new())
            }
            _ => Err(error),
        },
    }?;
    println!("Collected tasks.");
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("=>"))
        .after(after)
        .group(&GENERALCOMMANDS_GROUP);
    println!("Created framework.");
    let mut client = Client::builder(&config.bot_id)
        .framework(framework)
        .event_handler(Handler)
        .await?;
    println!("Created client.");
    // Get bot ID
    let bot_id = client
        .cache_and_http
        .http
        .get_current_application_info()
        .await?
        .id;
    // Cache bot ID
    client.data.write().await.insert::<BotIdKey>(bot_id);
    println!("Fetched and cached bot ID.");
    // Get all the roles in the guild to find the gulag role.
    let guild_roles = client
        .cache_and_http
        .http
        .get_guild_roles(config.guild_id.into())
        .await?;
    println!("Fetched guild roles.");
    // Try to find the gulag role.
    let gulag_role = guild_roles
        .into_iter()
        .find(|role| role.name == config.prisoner_role_name || role.id == config.prisoner_role_id)
        .ok_or({
            let msg = format!(
                "Failed to get gulag role by name ('{}') or ID ('{}').",
                config.prisoner_role_name, config.prisoner_role_id
            );
            IoError::new(IoErrorKind::InvalidData, msg.as_str())
        })?;
    println!("Found gulag role in guild roles.");
    // Once found, cache it.
    client.data.write().await.insert::<GulagRoleKey>(gulag_role);
    println!("Cached gulag role.");
    // Cache the tasks - they may need to be updated depending on role changes and such.
    client.data.write().await.insert::<TasksKey>(tasks);
    println!("Cached tasks.");
    // Create a channel for the bot thread to be able to send new tasks to the main thread.
    let (send, recv) = unbounded();
    client.data.write().await.insert::<TaskSenderKey>(send);
    println!("Cached gulag role and partial guild.");
    // Spawn a ctrl+c handler here and have it send the proper instructions n' stuff
    // todo
    // Configure the client
    /*client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("=>"))
            .cmd("gulag", Gulag)
            .cmd("current-gulags", CurrentGulags)
            .cmd("remove-gulag-info", RemoveGulagInfo)
            .cmd("anagram", Anagram)
            .cmd("source", Source)
            .cmd("help", Help),
    );*/
    println!("Starting client.");
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
    Ok(())
}
