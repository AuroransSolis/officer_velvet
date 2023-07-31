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
    fmt::{self, Debug, Display, Result as FmtResult},
    sync::Arc,
    time::Instant,
};

#[derive(Clone, Deserialize, Serialize)]
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
        self.end <= Utc::now()
    }

    pub async fn act(&self, data: &Arc<RwLock<TypeMap>>, http: &impl AsRef<Http>) -> AnyResult<()> {
        let start = Instant::now();
        println!("TL | GL | Getting context data reference.");
        let context_data = data.read().await;
        println!("TL | GL | Getting guild ID and prisoner role ID from cached config.");
        let Config {
            guild_id,
            prisoner_role_id,
            ..
        } = context_data.get::<ConfigKey>().unwrap();
        let guild_id = *guild_id.as_u64();
        let gulag_id = *prisoner_role_id.as_u64();
        println!(
            "TL | GL | Got guild ID {} and prisoner role ID {}",
            guild_id, gulag_id
        );
        println!("TL | GL | Getting member information.");
        let mut member = http
            .as_ref()
            .get_member(guild_id, self.user.1.into())
            .await?;
        println!("TL | GL | Removing prisoner role.");
        member.remove_role(http, gulag_id).await?;
        println!("TL | GL | Getting list of role IDs to add back to user.");
        let role_ids = self
            .roles
            .iter()
            .map(|&(_, role_id)| role_id)
            .collect::<Vec<_>>();
        println!("TL | GL | Adding roles back to user.");
        member.add_roles(http.as_ref(), &role_ids).await?;
        println!(
            "TL | GL | Successfully un-gulagged user in {:?}.",
            start.elapsed()
        );
        Ok(())
    }

    pub fn list_fmt(&self) -> String {
        format!(
            "  G | User \"{}\" ({}) until {}",
            self.user.0, self.user.1, self.end
        )
    }
}

impl Debug for Gulag {
    fn fmt(&self, f: &mut fmt::Formatter) -> FmtResult {
        writeln!(
            f,
            "Gulag entry for: '{}' (ID: {})",
            self.user.0, self.user.1
        )?;
        writeln!(f, "    Roles to restore:")?;
        for (role_name, role_id) in &self.roles {
            writeln!(f, "        - '{role_name}' (ID: {role_id})")?;
        }
        writeln!(f, "    End of sentence: {}", self.end)?;
        Ok(())
    }
}

impl Display for Gulag {
    fn fmt(&self, f: &mut fmt::Formatter) -> FmtResult {
        writeln!(
            f,
            "{} (ID: {}), release at <t:{end}> (<t:{end}:R>)",
            self.user.0,
            self.user.1,
            end = self.end.timestamp(),
        )
    }
}
