use serde::{Deserialize, Serialize};
use serenity::model::id::{GuildId, RoleId};
use std::default::Default;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub tasks_file: String,
    pub bot_id: String,
    pub guild_id: GuildId,
    pub prisoner_role_name: String,
    pub prisoner_role_id: RoleId,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            tasks_file: "tasks.json".into(),
            guild_id: 0.into(),
            bot_id: "bot key goes here".into(),
            prisoner_role_name: "Prisoner".into(),
            prisoner_role_id: 0.into(),
        }
    }
}
