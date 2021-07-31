use anyhow::Result as AnyResult;
use crossbeam_channel::{unbounded, Receiver as CbReceiver};
use serenity::{
    framework::{standard::macros::group, StandardFramework},
    http::client::Http,
    prelude::*,
};
use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind},
    sync::Arc,
    time::Duration,
};
use structopt::StructOpt;
use tokio::time::{interval, sleep};

mod anagram;
mod args;
mod cache_keys;
mod config;
mod current_gulags;
mod gulag;
mod handler;
mod help;
mod init;
mod leaderboard;
mod list_tasks;
mod misc;
mod release;
mod source;
mod tasks;

use anagram::*;
use cache_keys::*;
use config::Config;
use current_gulags::*;
use gulag::*;
use handler::{after, Handler};
use help::*;
use init::*;
use misc::*;
use release::*;
use source::*;
use tasks::*;

#[group]
#[commands(anagram, help, source)]
struct GeneralCommands;

#[group]
#[commands(create_task, current_gulags, gulag, release)]
struct AdminCommands;

pub const FOOTER_TEXT: &str = "Your friendly neighbourhood gulag officer, Officer Velvet";

#[tokio::main]
async fn main() -> AnyResult<()> {
    let args::Args { config_file_path } = args::Args::from_args();
    let config_contents = read_config_file(&config_file_path)?;
    println!("IN | Read config file contents.");
    let mut config = serde_json::from_str::<Config>(&config_contents)?;
    println!("IN | Parsed config from config file contents.");
    let tasks = read_tasks_file(&config)?;
    println!("IN | Collected tasks.");
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("=>"))
        .after(after)
        .group(&GENERALCOMMANDS_GROUP)
        .group(&ADMINCOMMANDS_GROUP);
    println!("IN | Created framework.");
    let mut client = Client::builder(&config.bot_id)
        .framework(framework)
        .event_handler(Handler)
        .await?;
    println!("IN | Created client.");
    // Get bot ID
    let bot_id = client
        .cache_and_http
        .http
        .get_current_application_info()
        .await?
        .id;
    // Cache bot ID
    client.data.write().await.insert::<BotIdKey>(bot_id);
    println!("IN | Fetched and cached bot ID.");
    // Get all the roles in the guild to find the gulag role.
    let guild_roles = client
        .cache_and_http
        .http
        .get_guild_roles(config.guild_id.into())
        .await?;
    println!("IN | Fetched guild roles.");
    // Try to find the gulag role.
    let gulag_role = find_role_by(
        &guild_roles,
        |&role| role.id == config.prisoner_role_id || role.name == config.prisoner_role_name,
        || {
            let msg = format!(
                "IN | Failed to get gulag role by name ('{}') or ID ('{}').",
                config.prisoner_role_name, config.prisoner_role_id
            );
            IoError::new(IoErrorKind::InvalidData, msg.as_str()).into()
        },
    )?;
    println!("IN | Found gulag role in guild roles.");
    println!("IN | Checking whether it is necessary to update the prisoner role name or ID");
    // Update role name and/or ID in config if necessary, and write out to file.
    update_config_if(
        &config_file_path,
        &mut config,
        |config| {
            gulag_role.id != config.prisoner_role_id || gulag_role.name != config.prisoner_role_name
        },
        |config| {
            if gulag_role.id != config.prisoner_role_id {
                println!("IN | CF | IDs do not match. Updating ID.");
                config.prisoner_role_id = gulag_role.id;
            } else {
                println!("IN | CF | Names do not match. Updating name.");
                config.prisoner_role_name.clear();
                config.prisoner_role_name.push_str(&gulag_role.name);
            }
        },
    )?;
    // Cache gulag role.
    client.data.write().await.insert::<GulagRoleKey>(gulag_role);
    println!("IN | Cached gulag role.");
    // Try to find the Nitro role.
    let nitro_role = find_role_by(
        &guild_roles,
        |&role| role.id == config.nitro_role_id || role.name == config.nitro_role_name,
        || {
            let msg = format!(
                "IN | Failed to get Nitro role by name ('{}') or ID ('{}').",
                config.prisoner_role_name, config.prisoner_role_id
            );
            IoError::new(IoErrorKind::InvalidData, msg.as_str()).into()
        },
    )?;
    println!("IN | Found Nitro role in guild roles.");
    println!("IN | Checking whether it is necessary to update the Nitro role name or ID");
    // Update role name and/or ID in config if necessary, and write out to file.
    update_config_if(
        &config_file_path,
        &mut config,
        |config| nitro_role.id != config.nitro_role_id || nitro_role.name != config.nitro_role_name,
        |config| {
            if nitro_role.id != config.nitro_role_id {
                println!("IN | CF | IDs do not match. Updating ID.");
                config.nitro_role_id = nitro_role.id;
            } else {
                println!("IN | CF | Names do not match. Updating name.");
                config.nitro_role_name.clear();
                config.nitro_role_name.push_str(&nitro_role.name);
            }
        },
    )?;
    // Cache Nitro role.
    client.data.write().await.insert::<NitroRoleKey>(nitro_role);
    println!("IN | Cached Nitro role.");
    // Find all the roles allowed permission to use all commands and cache them as well.
    let elevated_roles = guild_roles
        .iter()
        .filter(|&role1| {
            config
                .elevated_roles
                .iter()
                .any(|role2| &role1.id == &role2.1 || &role1.name == &role2.0)
        })
        .map(|role| role.clone())
        .collect::<Vec<_>>();
    println!("IN | Found elevated roles.");
    println!("IN | Checking whether it is necessary to update elevated role names or IDs");
    // Update role name and/or ID for each role in config if necessary, and write out to file.
    update_config_if(
        &config_file_path,
        &mut config,
        |config| {
            elevated_roles.iter().any(|role| {
                config
                    .elevated_roles
                    .iter()
                    .any(|(name, id)| (name != role.name.as_str()) ^ (*id != role.id))
            })
        },
        |config| {
            elevated_roles.iter().for_each(|role| {
                println!(
                    "IN | CF | Checking config values for role '{}' (ID {})",
                    role.name, role.id
                );
                config.elevated_roles.iter_mut().for_each(|(name, id)| {
                    let matching_ids = *id == role.id;
                    let matching_names = name == role.name.as_str();
                    if !matching_ids & matching_names {
                        println!("IN | CF | IDs do not match. Updating ID.");
                        *id = role.id;
                    } else if matching_ids & !matching_names {
                        println!("IN | CF | Names do not match. Updating name.");
                        name.clear();
                        name.push_str(role.name.as_str());
                    } else {
                        println!("IN | CF | Name and ID match.");
                    }
                })
            })
        },
    )?;
    // Cache elevated roles.
    client
        .data
        .write()
        .await
        .insert::<ElevatedRolesKey>(elevated_roles);
    // Cache the config.
    client.data.write().await.insert::<ConfigKey>(config);
    // Cache the tasks - they may need to be updated depending on role changes and such.
    client.data.write().await.insert::<TasksKey>(tasks);
    println!("IN | Cached tasks.");
    // Create a channel for the bot thread to be able to send new tasks to the main thread.
    let (send, recv) = unbounded();
    client.data.write().await.insert::<TaskSenderKey>(send);
    // Spawn a ctrl+c handler here and have it send the proper instructions n' stuff.
    // todo
    // Start the task handling loop in a separate thread.
    println!("IN | Starting task handling loop.");
    let data_clone = client.data.clone();
    let http_clone = client.cache_and_http.http.clone();
    tokio::spawn(start_task_handler(data_clone, http_clone, recv));
    // Start the client.
    println!("IN | Starting client.");
    if let Err(why) = client.start().await {
        println!("IN | Client error: {:?}", why);
    }
    Ok(())
}

async fn start_task_handler(
    data: Arc<RwLock<TypeMap>>,
    http: Arc<Http>,
    recv: CbReceiver<TaskType>,
) -> AnyResult<()> {
    let mut interval = interval(Duration::from_millis(1000));
    let tasklist_filename = data
        .read()
        .await
        .get::<ConfigKey>()
        .unwrap()
        .tasks_file
        .clone();
    loop {
        // Get copy of task list.
        let mut tasks = data.read().await.get::<TasksKey>().unwrap().clone();
        let mut made_changes = false;
        // Check for new tasks.
        while let Ok(task) = recv.try_recv() {
            println!("TL | Received new task - pushing to task list.");
            tasks.push(task);
            made_changes = true;
        }
        // Check whether any current tasks need to be executed.
        for i in (0..tasks.len()).rev() {
            if tasks[i].time_to_act() {
                tasks[i].act(&data, &http).await?;
                if tasks[i].is_gulag() {
                    println!("TL | Gulag period has elapsed - removing from task list.");
                    tasks.remove(i);
                    made_changes = true;
                }
            }
        }
        if made_changes {
            println!("TL | Changes to task list were made.");
            update_task_list(&tasklist_filename, &tasks).await?;
            // Sync global tasklist with new list.
            *data.write().await.get_mut::<TasksKey>().unwrap() = tasks;
        }
        interval.tick().await;
    }
}
