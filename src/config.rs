use serde::{Deserialize, Serialize};
use serenity::model::id::{GuildId, RoleId};
use std::default::Default;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub tasks_file: String,
    pub bot_id: String,
    pub icon_url: String,
    pub guild_id: GuildId,
    pub elevated_roles: Vec<(String, RoleId)>,
    pub prisoner_role_name: String,
    pub prisoner_role_id: RoleId,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            tasks_file: "".into(),
            bot_id: "".into(),
            icon_url: "".into(),
            guild_id: 0.into(),
            elevated_roles: Vec::new(),
            prisoner_role_name: "".into(),
            prisoner_role_id: 0.into(),
        }
    }
}
