use crate::cache_keys::ConfigKey;
use anyhow::{anyhow, Result as AnyResult};
use serde::{Deserialize, Serialize};
use serenity::{client::Cache, http::{CacheHttp, Http}, model::id::{ChannelId, GuildId, RoleId, UserId}, prelude::{RwLock, TypeMap}};
use std::{sync::Arc, time::Duration};

mod scoring_method;

use scoring_method::ScoringMethod;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaderboardEntry {
    pub name: String,
    pub id: UserId,
    pub score: usize,
}

impl LeaderboardEntry {
    pub fn new(name: String, id: UserId, score: usize) -> Self {
        LeaderboardEntry { name, id, score }
    }
}

// serde is a temporary stand-in while I learn DB stuff.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Leaderboard {
    pub leaderboard: Vec<LeaderboardEntry>,
    pub number: usize,
    pub guild: GuildId,
    pub scoring_method: ScoringMethod,
    pub update_period: Duration,
    pub award: Option<(String, RoleId)>,
    pub last_given_to: Option<UserId>,
}

impl Leaderboard {
    async fn update(&mut self, cache_http: &impl CacheHttp) -> AnyResult<()> {
        let untrimmed_leaderboard = self.scoring_method.get_leaderboard(http).await?;
        self.leaderboard = untrimmed_leaderboard
            .into_iter()
            .take(self.number)
            .collect();
        Ok(())
    }
 
    async fn award_winner(
        &mut self,
        cache_http: &impl CacheHttp,
    ) -> AnyResult<()> {
        if let Some((role_name, role_id)) = &self.award {
            println!(
                "LB | AW | Attempting to award role '{}' to user '{}' ({})",
                role_name, self.leaderboard[0].name, self.leaderboard[0].id
            );
            let award_to = self.leaderboard[0].id;
            println!("LB | AW | Getting guild ID");
            let guild_id = cache_http.cache().read().await.get::<ConfigKey>().unwrap().guild_id;
            if let Some(last) = self.last_given_to {
                println!("LB | AW | No awards given previously.");
                
            }
        }
        Ok(())
    }
}

async fn try_give_role(http: &impl AsRef<Http>, cache: &Arc<Cache>, guild: GuildId, user: UserId, role: RoleId) -> AnyResult<()> {
    let member = guild.member((cache, http.as_ref()), user).await?;
    if member.roles.contains(&role) {
        println!("LB | AW | User already has role ID {}", role);
        Ok(())
    } else {

    }
}
