use hyper::body::Bytes;
use serde::Deserialize;

use crate::client::{
    events::{BlockMessageInteractionEvent, Events},
    Channel, User, EVENTS,
};

#[derive(Deserialize, Debug)]
struct InteractionPayload {
    #[serde(rename = "type")]
    pub ty: String,
    pub trigger_id: String,
    pub user: RawUser,
    pub channel: RawChannel,
    pub actions: Option<Vec<RawAction>>,
    pub container: RawContainer,
    //pub user: String,
}

#[derive(Deserialize, Debug)]
struct RawUser {
    pub id: String,
    pub username: String,
    pub name: String,
    // TODO: team id
}

#[derive(Deserialize, Debug)]
struct RawChannel {
    pub id: String,
    #[serde(rename = "name")]
    pub ty: String,
}

#[derive(Deserialize, Debug)]
struct RawAction {
    action_id: String,
    block_id: String,
    #[serde(rename = "value")]
    action: String,
}

#[derive(Deserialize, Debug)]
struct RawContainer {
    #[serde(rename = "type")]
    ty: String,
}

#[derive(Deserialize, Debug)]
struct RawMessage {
    ts: String,
}

pub(super) async fn handle_interaction(content: String) -> Bytes {
    let interaction: InteractionPayload = serde_json::from_str(&content).unwrap();

    println!();
    println!();
    //println!("{}", content);

    println!("{}", content);
    println!("{:#?}", interaction);

    if interaction.container.ty == "message" {
        EVENTS
            .get()
            .unwrap()
            .send(Events::BlockMessageInteraction(
                BlockMessageInteractionEvent {
                    // how this work?
                    action: interaction.actions.unwrap()[0].action.clone(),
                    trigger_id: interaction.trigger_id,
                    user: User::new(interaction.user.id),
                    username: interaction.user.username,
                    display_name: interaction.user.name,
                    channel: Channel::new(interaction.channel.id),
                },
            ))
            .await
            .unwrap();
    }

    Bytes::from("")
}
