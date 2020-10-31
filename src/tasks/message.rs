use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serenity::{builder::CreateMessage, utils::Colour};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MessageType {
    Plain {
        content: String,
    },
    Embed {
        author_icon_url: Option<String>,
        author_name: Option<String>,
        author_url: Option<String>,
        colour: (u8, u8, u8),
        description: Option<String>,
        fields: Option<Vec<(String, String, bool)>>,
        footer_text: Option<String>,
        image_url: Option<String>,
        thumbnail_url: Option<String>,
        timestamp: bool,
        title: Option<String>,
        title_url: Option<String>,
    },
}

impl MessageType {
    pub fn is_embed(&self) -> bool {
        match self {
            MessageType::Embed { .. } => true,
            _ => false,
        }
    }

    pub fn build(&self) -> CreateMessage {
        let mut message = CreateMessage::default();
        match self {
            MessageType::Plain { content } => {
                message.content(content);
            }
            MessageType::Embed {
                author_icon_url,
                author_name,
                author_url,
                colour: (r, g, b),
                description,
                fields,
                footer_text,
                image_url,
                thumbnail_url,
                timestamp,
                title,
                title_url,
            } => {
                message.embed(|e| {
                    author_icon_url
                        .as_ref()
                        .map(|url| e.author(|auth| auth.icon_url(url)));
                    author_name
                        .as_ref()
                        .map(|name| e.author(|auth| auth.name(name)));
                    author_url
                        .as_ref()
                        .map(|url| e.author(|auth| auth.url(url)));
                    e.colour(Colour::from_rgb(*r, *g, *b));
                    description.as_ref().map(|desc| e.description(desc));
                    fields.as_ref().map(|vec| e.fields(vec.clone()));
                    footer_text.as_ref().map(|text| e.footer(|f| f.text(text)));
                    image_url.as_ref().map(|url| e.image(url));
                    thumbnail_url.as_ref().map(|url| e.thumbnail(url));
                    if *timestamp {
                        e.timestamp(Utc::now().to_string());
                    }
                    title.as_ref().map(|title| e.title(title));
                    title_url.as_ref().map(|url| e.url(url));
                    e
                });
            }
        }
        message
    }
}
