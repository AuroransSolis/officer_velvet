use serde::{Deserialize, Serialize};
use serenity::model::id::{GuildId, RoleId};
use std::default::Default;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub tasks_file: String,
    pub bot_id: String,
    pub files_dir: String,
    pub icon_filename: String,
    pub leaderboard_filename: String,
    pub guild_id: GuildId,
    pub elevated_roles: Vec<(String, RoleId)>,
    pub prisoner_role_name: String,
    pub prisoner_role_id: RoleId,
    pub nitro_role_name: String,
    pub nitro_role_id: RoleId,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            tasks_file: "".into(),
            bot_id: "".into(),
            files_dir: "files".into(),
            icon_filename: "default.png".into(),
            leaderboard_filename: "leaderboard".into(),
            guild_id: 0.into(),
            elevated_roles: Vec::new(),
            prisoner_role_name: "".into(),
            prisoner_role_id: 0.into(),
            nitro_role_name: "".into(),
            nitro_role_id: 0.into(),
        }
    }
}
