use crate::{config::Config, tasks::TaskType};
use anyhow::{Error as AnyError, Result as AnyResult};
use serenity::model::guild::Role;
use std::{
    fs::{self, File},
    io::{ErrorKind as IoErrorKind, Write},
};

pub fn read_config_file(cf: &str) -> AnyResult<String> {
    match fs::read_to_string(cf) {
        Ok(contents) => Ok(contents),
        Err(error) => {
            if error.kind() == IoErrorKind::NotFound {
                println!(
                    "\
                        IN | Config file not found. \
                        Attempting to create new default config file at '{cf}'\
                    "
                );
                let mut new_config_file = File::create(cf)?;
                let default_contents = serde_json::to_string_pretty(&Config::default()).unwrap();
                new_config_file.write_all(default_contents.as_bytes())?;
                println!("IN | Created new config file and wrote defaults.");
            }
            Err(error.into())
        }
    }
}

pub fn read_tasks_file(config: &Config) -> AnyResult<Vec<TaskType>> {
    match fs::read_to_string(&config.tasks_file) {
        Ok(contents) if contents.is_empty() => Ok(Vec::new()),
        Ok(contents) => Ok(serde_json::from_str::<Vec<TaskType>>(&contents)?),
        Err(error) => match error.kind() {
            IoErrorKind::NotFound => {
                println!(
                    "IN | Tasks file not found. Attempting to create new tasks file at '{}'",
                    config.tasks_file
                );
                let _ = File::create(&config.tasks_file)?;
                println!("IN | Created new blank tasks file.");
                Ok(Vec::new())
            }
            _ => Err(error.into()),
        },
    }
}

pub fn find_role_by<Find: FnMut(&&Role) -> bool, Err: FnOnce() -> AnyError>(
    roles: &[Role],
    find_by: Find,
    err: Err,
) -> AnyResult<Role> {
    roles.iter().find(find_by).cloned().ok_or_else(err)
}

pub fn update_config_if<Condition: FnOnce(&Config) -> bool, UpdateConfig: FnOnce(&mut Config)>(
    filename: &str,
    config: &mut Config,
    condition: Condition,
    update_config: UpdateConfig,
) -> AnyResult<()> {
    if condition(config) {
        update_config(config);
        println!("IN | CF | Re-creating config file.");
        let mut file = File::create(filename)?;
        println!("IN | CF | Serializing updated config.");
        let config_string = serde_json::to_string_pretty(config)?;
        println!("IN | CF | Writing updated config to file.");
        file.write_all(config_string.as_bytes())?;
        println!("IN | CF | Updated saved config.");
        Ok(())
    } else {
        println!("IN | CF | Saved config is up to date.");
        Ok(())
    }
}
