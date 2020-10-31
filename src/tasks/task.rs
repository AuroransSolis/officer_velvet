use super::message::MessageType;
use crate::{cache_keys::ConfigKey, FILES_DIR};
use anyhow::Result as AnyResult;
use serde::{Deserialize, Serialize};
use serenity::{
    http::client::Http,
    model::id::ChannelId,
    prelude::{RwLock, TypeMap},
};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Task {
    SendMessage {
        send_to: ChannelId,
        message: MessageType,
        upload_file: Option<String>,
    },
    UpdateAppearance {
        new_name: String,
        new_icon_url: String,
    },
}

impl Task {
    pub async fn act(&self, data: &Arc<RwLock<TypeMap>>, http: &Arc<Http>) -> AnyResult<()> {
        match self {
            Task::SendMessage {
                send_to,
                message,
                upload_file,
                ..
            } => {
                let mut create_message = message.build();
                if message.is_embed() {
                    let context_data = data.read().await;
                    let icon_url = context_data.get::<ConfigKey>().unwrap().icon_url.as_str();
                    create_message.embed(|embed| embed.footer(|footer| footer.icon_url(icon_url)));
                }
                if let Some(filename) = upload_file.as_ref() {
                    let mut base = PathBuf::from(FILES_DIR);
                    base.push(filename.as_str());
                    create_message.add_file(base.as_path());
                    let _ = send_to.send_message(&http, |_| &mut create_message).await?;
                } else {
                    let _ = send_to.send_message(&http, |_| &mut create_message).await?;
                }
            }
            Task::UpdateAppearance {
                new_name,
                new_icon_url,
                ..
            } => {
                let context_data = data.read().await;
                let guild_id = context_data.get::<ConfigKey>().unwrap().guild_id;
                http.edit_nickname(guild_id.into(), Some(new_name.as_str()))
                    .await?;
                http.get_current_user()
                    .await?
                    .edit(&http, |user| user.avatar(Some(new_icon_url.as_str())))
                    .await?;
            }
        }
        Ok(())
    }
}
