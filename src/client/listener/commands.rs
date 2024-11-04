use hyper::body::Bytes;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::client::{
    events::{Events, SlashCommandCalledEvent},
    Channel, Context, SlackApiResponse, User, EVENTS,
};

#[derive(Deserialize, Debug)]
pub struct CommandPayload {
    pub command: String,
    pub text: String,
    pub trigger_id: String,
    pub user_id: String,
    pub channel_id: String,
    pub team_id: String,
    pub response_url: String,
}

pub(super) async fn handle_command(content: String) -> Bytes {
    let command: CommandPayload = serde_html_form::from_str(&content).unwrap();

    EVENTS
        .get()
        .unwrap()
        .send(Events::SlashCommandCalled(SlashCommandCalledEvent {
            command: command.command,
            text: command.text,
            trigger_id: command.trigger_id,
            user: User::new(command.user_id),
            channel: Channel::new(command.channel_id),
        }))
        .await
        .unwrap();

    //println!("{:#?}", command);

    Bytes::from("")
}
