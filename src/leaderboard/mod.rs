use anyhow::{anyhow, Result as AnyResult};
use serde::{Deserialize, Serialize};
use serenity::{
    http::Http,
    model::id::{ChannelId, RoleId, UserId},
    prelude::{RwLock, TypeMap},
};
use std::{sync::Arc, time::Duration};

use crate::cache_keys::ConfigKey;

mod pings;
mod pins;

/*
Honestly not sure about any of this. I probably should cache leaderboards and update them on
certain events, but I'm not sure how to do that somewhat generically without massively specializing.
*/

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LeaderboardType {
    Pins {
        name: String,
        current_id: ChannelId,
        archive_to: ChannelId,
    },
    Pings {
        channel_id: ChannelId,
        track_for: Duration,
    },
}

impl LeaderboardType {
    async fn channel_id(
        &mut self,
        http: Arc<Http>,
        data: Arc<RwLock<TypeMap>>,
    ) -> AnyResult<ChannelId> {
        match self {
            LeaderboardType::Pings { channel_id, .. } => Ok(*channel_id),
            LeaderboardType::Pins {
                ref name,
                current_id: last_id,
                archive_to,
            } => {
                let guild_id = data.read().await.get::<ConfigKey>().unwrap().guild_id;
                let current_id = http
                    .get_channels(*guild_id.as_u64())
                    .await?
                    .into_iter()
                    .find(|channel| channel.name.as_str() == name.as_str())
                    .ok_or_else(|| {
                        anyhow!(
                            "Channel with name '{}' doesn't exist for guild ID {}",
                            name,
                            guild_id
                        )
                    })?;
                if *last_id != current_id.id {
                    *last_id = current_id.id;
                }
                Ok(current_id.id)
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Leaderboard {
    pub lb_type: LeaderboardType,
    pub award: Option<(String, RoleId)>,
    pub given_to: Vec<UserId>,
}

impl Leaderboard {
    async fn update(&mut self) -> AnyResult<()> {
        Ok(())
    }
}
