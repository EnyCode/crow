use serde::Serialize;

use super::{channel::Channel, user::User};

// TODO: add more stuff to this
// TODO: make pub(super) stuff more public? theres no reason for it not to be
// TODO: make a macro for the #[serde(skip_serializing_if = "Option::is_none")] bits
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Serialize)]
pub struct Message {
    pub channel: Option<Channel>,
    #[serde(flatten)]
    pub content: MessageContent,

    // non user stuff, but needed for json
    thread_ts: Option<String>,
    #[serde(rename = "user")]
    pub(super) receiver: Option<User>,
}

impl Message {
    pub fn new() -> MessageBuilder {
        MessageBuilder {
            channel: None,
            content: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageContent {
    Text(String),
    Blocks(String),
}

impl Default for MessageContent {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

impl From<&str> for Message {
    fn from(text: &str) -> Self {
        Message {
            channel: None,
            content: MessageContent::Text(text.to_string()),
            thread_ts: None,
            receiver: None,
        }
    }
}

pub struct MessageBuilder {
    pub channel: Option<String>,
    pub content: Option<MessageContent>,
}

impl MessageBuilder {
    pub fn channel_id(mut self, channel: String) -> Self {
        self.channel = Some(channel);
        self
    }

    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = Some(channel.id);
        self
    }

    pub fn text(mut self, text: String) -> Self {
        self.content = Some(MessageContent::Text(text));
        self
    }

    pub fn blocks(mut self, blocks: String) -> Self {
        self.content = Some(MessageContent::Blocks(blocks));
        self
    }

    pub fn build(self) -> Message {
        Message {
            channel: self.channel.map(|id| Channel { id }),
            content: self.content.unwrap(),
            thread_ts: None,
            receiver: None,
        }
    }
}
