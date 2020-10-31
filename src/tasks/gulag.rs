use crate::{cache_keys::ConfigKey, config::Config};
use anyhow::Result as AnyResult;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serenity::{
    http::client::Http,
    model::id::{RoleId, UserId},
    prelude::{RwLock, TypeMap},
};
use std::{
    fmt::{self, Display, Result as FmtResult},
    sync::Arc,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Gulag {
    pub user: (String, UserId),
    pub roles: Vec<(String, RoleId)>,
    pub end: DateTime<Utc>,
}

impl Gulag {
    pub fn new(user: (String, UserId), roles: Vec<(String, RoleId)>, end: DateTime<Utc>) -> Self {
        Gulag { user, roles, end }
    }

    pub fn time_to_act(&self) -> bool {
        self.end >= Utc::now()
    }

    pub async fn act(&self, data: &Arc<RwLock<TypeMap>>, http: &Arc<Http>) -> AnyResult<()> {
        let context_data = data.read().await;
        let Config {
            guild_id,
            prisoner_role_id,
            ..
        } = context_data.get::<ConfigKey>().unwrap();
        let guild_id = *guild_id.as_u64();
        let gulag_id = *prisoner_role_id.as_u64();
        http.remove_member_role(guild_id, self.user.1.into(), gulag_id)
            .await?;
        for &(_, role_id) in &self.roles {
            http.add_member_role(guild_id, self.user.1.into(), role_id.into())
                .await?;
        }
        Ok(())
    }
}

impl Display for Gulag {
    fn fmt(&self, f: &mut fmt::Formatter) -> FmtResult {
        writeln!(
            f,
            "Gulag entry for: '{}' (ID: {})",
            self.user.0, self.user.1
        )?;
        writeln!(f, "    Roles to restore:")?;
        for (role_name, role_id) in &self.roles {
            writeln!(f, "        - '{}' (ID: {})", role_name, role_id)?;
        }
        writeln!(f, "    End of sentence: {}", self.end)?;
        Ok(())
    }
}
