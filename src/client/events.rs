#[derive(Debug, Clone)]
pub enum Events {
    MemberJoinedChannel(MemberJoinedChannelEvent),
    Callback,
}

#[derive(Debug, Clone)]
pub struct MemberJoinedChannelEvent {
    pub channel: String,
    pub user: String,
    pub inviter: String,
}
