use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serenity::model::id::{RoleId, UserId};

#[derive(Debug, Deserialize, Serialize)]
pub struct Gulag {
    pub user: UserId,
    pub roles: Vec<(String, RoleId)>,
    pub end: DateTime<Utc>,
}

impl Gulag {
    pub fn time_to_act(&self) -> bool {
        self.end >= Utc::now()
    }
}
