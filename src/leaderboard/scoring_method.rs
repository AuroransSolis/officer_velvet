use std::collections::HashMap;

use super::LeaderboardEntry;
use anyhow::Result as AnyResult;
use chrono::{Duration, Utc, DateTime};
use serde::{Deserialize, Serialize};
use serenity::{
    futures::StreamExt,
    http::Http,
    model::{channel::Message, id::ChannelId, prelude::User},
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ScoringMethod {
    PinsIn(ChannelId),
    PingsIn {
        channel: ChannelId,
        within_days: i64,
    },
}

impl ScoringMethod {
    pub async fn get_leaderboard(
        &self,
        http: &impl AsRef<Http>,
    ) -> AnyResult<Vec<LeaderboardEntry>> {
        match self {
            ScoringMethod::PinsIn(channel) => {
                let messages = channel.pins(http).await?;
                Ok(pin_list_to_scoreboard(messages))
            }
            ScoringMethod::PingsIn {
                channel,
                within_days,
            } => {
                let mut messages = Vec::new();
                let mut stream = channel.messages_iter(http).boxed_local();
                while let Some(msg_result) = stream.next().await {
                    let msg = msg_result?;
                    let timestamp = msg.timestamp;
                    let naive_dt = timestamp.naive_utc();
                    let dt: DateTime<Utc> = DateTime::from_utc(naive_dt, Utc);
                    if Utc::now().signed_duration_since(dt)
                        >= Duration::days(*within_days)
                        && !msg.mentions.is_empty()
                    {
                        messages.push(msg);
                    } else {
                        break;
                    }
                }
                Ok(ping_list_to_scoreboard(messages))
            }
        }
    }
}

fn pin_list_to_scoreboard(messages: Vec<Message>) -> Vec<LeaderboardEntry> {
    let mut score_map = HashMap::new();
    for message in messages {
        let Message {
            author:
                User {
                    id: author_id,
                    name: author_name,
                    ..
                },
            ..
        } = message;
        score_map
            .entry((author_name, author_id))
            .and_modify(|score| *score += 1)
            .or_insert(1);
    }
    let mut leaderboard = score_map
        .into_iter()
        .map(|((name, id), score)| LeaderboardEntry::new(name, id, score))
        .collect::<Vec<_>>();
    leaderboard.sort_unstable_by(|e0, e1| e0.score.cmp(&e1.score));
    leaderboard
}

fn ping_list_to_scoreboard(messages: Vec<Message>) -> Vec<LeaderboardEntry> {
    let mut score_map = HashMap::new();
    for message in messages {
        for mention in message.mentions {
            let User { id, name, .. } = mention;
            score_map
                .entry((name, id))
                .and_modify(|score| *score += 1)
                .or_insert(1);
        }
    }
    let mut leaderboard = score_map
        .into_iter()
        .map(|((name, id), score)| LeaderboardEntry::new(name, id, score))
        .collect::<Vec<_>>();
    leaderboard.sort_unstable_by(|e0, e1| e0.score.cmp(&e1.score));
    leaderboard
}
