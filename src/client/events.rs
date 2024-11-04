use tokio::sync::mpsc::Sender;

use super::{Channel, User};

#[derive(Debug, Clone)]
pub enum Events {
    MemberJoinedChannel(MemberJoinedChannelEvent),
    SlashCommandCalled(SlashCommandCalledEvent),
    BlockMessageInteraction(BlockMessageInteractionEvent),
    Callback,
}

// TODO: channel & user structs
#[derive(Debug, Clone)]
pub struct MemberJoinedChannelEvent {
    pub channel: Channel,
    pub user: User,
    pub inviter: Option<User>,
}

#[derive(Debug, Clone)]
pub struct SlashCommandCalledEvent {
    pub command: String,
    pub text: String,
    pub user: User,
    pub channel: Channel,
    pub trigger_id: String,
}

#[derive(Debug, Clone)]
pub struct BlockMessageInteractionEvent {
    pub action: String,
    pub trigger_id: String,
    pub user: User,
    pub username: String,
    pub display_name: String,
    pub channel: Channel,
}
