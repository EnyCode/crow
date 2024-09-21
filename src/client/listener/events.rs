use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockWriteGuard},
};

use hyper::body::Bytes;
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

use crate::client::{
    events::{Events, MemberJoinedChannelEvent},
    EVENTS,
};

// umm how much of this is important?
#[derive(Deserialize, Debug)]
pub struct GenericEvent {
    //pub r#type: String,
    //pub token: String,
    //pub team_id: String,
    //pub api_app_id: String,
    pub event: HashMap<String, String>,
    //pub event_context: String,
    //pub event_id: String,
    //pub event_time: u32,
    //pub authorizations:
    //pub is_ext_shared_channel: bool,
    //pub context_team_id: String,
    // another thing goes here about enterprises that is null
}

pub(super) async fn handle_event(content: String) -> Bytes {
    let event = match serde_json::from_str::<GenericEvent>(&content) {
        Ok(event) => event,
        Err(err) => {
            println!("error: {:?}", err);
            return Bytes::from("Invalid request");
        }
    };

    println!("event: {:?}", event);

    match event.event.get("type").unwrap().as_str() {
        "member_joined_channel" => {
            let ev = match serde_json::from_str::<MemberJoinedChannel>(
                format!("{:?}", event.event).as_str(),
            ) {
                Ok(member_joined_channel) => member_joined_channel,
                Err(_) => {
                    return Bytes::from("Invalid request");
                }
            };

            println!("member_joined_channel: <@{}>", ev.user);

            EVENTS
                .get()
                .unwrap()
                .send(Events::MemberJoinedChannel(MemberJoinedChannelEvent {
                    channel: ev.channel,
                    user: ev.user,
                    inviter: ev.inviter.unwrap_or("".to_string()),
                }))
                .await
                .unwrap();

            return Bytes::from("200 OK");
        }
        _ => {
            return Bytes::from("Invalid request");
        }
    }
}

// EVENTS

#[derive(Deserialize, Debug)]
struct MemberJoinedChannel {
    pub user: String,
    pub event_ts: String,
    pub channel: String,
    pub channel_type: String,
    pub team: String,
    pub inviter: Option<String>,
    pub enterprise: Option<String>,
}
