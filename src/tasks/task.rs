use super::message::MessageType;
use crate::cache_keys::ConfigKey;
use anyhow::Result as AnyResult;
use serde::{Deserialize, Serialize};
use serenity::{
    http::client::Http,
    model::id::ChannelId,
    prelude::{RwLock, TypeMap},
    utils::read_image,
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
        new_icon_filename: String,
    },
}

impl Task {
    pub async fn act(&self, data: &Arc<RwLock<TypeMap>>, http: &impl AsRef<Http>) -> AnyResult<()> {
        match self {
            Task::SendMessage {
                send_to,
                message,
                upload_file,
            } => {
                let mut create_message = message.build();
                if message.is_embed() {
                    let icon_url = http
                        .as_ref()
                        .get_current_user()
                        .await?
                        .avatar_url()
                        .unwrap();
                    create_message.embed(|embed| embed.footer(|footer| footer.icon_url(icon_url)));
                }
                if let Some(filename) = upload_file.as_ref() {
                    let files_dir = data
                        .read()
                        .await
                        .get::<ConfigKey>()
                        .unwrap()
                        .files_dir
                        .clone();
                    let mut base = PathBuf::from(&files_dir);
                    base.push(filename.as_str());
                    create_message.add_file(base.as_path());
                    let _ = send_to.send_message(&http, |_| &mut create_message).await?;
                } else {
                    let _ = send_to.send_message(&http, |_| &mut create_message).await?;
                }
            }
            Task::UpdateAppearance {
                new_name,
                new_icon_filename,
            } => {
                let (guild_id, files_dir) = {
                    let context = data.read().await;
                    let config = context.get::<ConfigKey>().unwrap();
                    (config.guild_id, config.files_dir.clone())
                };
                http.as_ref()
                    .edit_nickname(guild_id.into(), Some(new_name.as_str()))
                    .await?;
                let avatar_b64 = read_image(format!(
                    "{}/{}",
                    files_dir.trim_end_matches('/'),
                    new_icon_filename
                ))?;
                http.as_ref()
                    .get_current_user()
                    .await?
                    .edit(&http, |user| user.avatar(Some(&avatar_b64)))
                    .await?;
            }
        }
        Ok(())
    }

    pub fn list_fmt(&self) -> &str {
        match self {
            Task::SendMessage { .. } => "   SEND",
            Task::UpdateAppearance { .. } => "UPDATE",
        }
    }
}

impl Default for Task {
    fn default() -> Self {
        Task::SendMessage {
            send_to: 0.into(),
            message: MessageType::Plain {
                content: String::new(),
            },
            upload_file: None,
        }
    }
}
