use serde::{Deserialize, Serialize};
use serenity::model::id::{ChannelId, UserId};

mod pings;
mod pins;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum LeaderboardType {
    Pins,
    Pings,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Leaderboard {
    lb_type: LeaderboardType,
    watch_channel: ChannelId,
    lb_entries: Vec<(UserId, String, usize)>,
}
