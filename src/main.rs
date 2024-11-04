use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;

use chrono::TimeZone;
use chrono::{Local, Timelike, Utc};
use chrono_tz::Europe::London;
use client::events::BlockMessageInteractionEvent;
use client::interactions::{CommandFn, InteractionContext, SlashCommand};
use client::{Channel, Context, EventHandler, Message, SlackClient, User};
use cron::Cron;
use crow_derive::{message_action, slash_command};
use futures::future::BoxFuture;
use futures::FutureExt;

#[macro_use]
mod client;
mod cron;

pub struct CronEvent {
    cron: Cron,
    channel: Channel,
    message: Message,
}

impl CronEvent {
    pub fn new(cron: &str, channel: &str, message: Message) -> Self {
        Self {
            cron: Cron::parse(cron).unwrap(),
            channel: Channel {
                id: channel.to_string(),
            },
            message,
        }
    }
}

struct Handler {
    crons: Vec<CronEvent>,
}

impl EventHandler for Handler {
    async fn member_joined_channel(
        &self,
        ctx: Context<'_>,
        channel: Channel,
        user: User,
        _inviter: Option<User>,
    ) {
        channel
            .post_message(
                &ctx,
                Message::new()
                    .text(format!(
                        "<@U078VGCHXSP>, welcome <@{}> to the cove!",
                        user.id
                    ))
                    .build(),
                Some(User {
                    id: "U078VGCHXSP".to_string(),
                }),
            )
            .await
            .unwrap();

        user.send_dm(
            &ctx,
            Message::new()
                .text("Arrr, *welcome to the cove, landlubber*! Ye be joining a place fer only the _boldest o' buccaneers_!".to_string())
                .build(),
        ).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        user.send_dm(
            &ctx,
            Message::new()
                .text("However, this be no ordinary hideaway, matey! Here we gather our *plundered booty*, swap *tales o' the high seas* an' *revel in our victories* o'er many a foe!".to_string())
                .build(),
        ).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        user.send_dm(
            &ctx,
            Message::new()
                .blocks(r#"[{"type": "section","text": {"type": "mrkdwn","text": "Now, throw off yer landlubber ways, grab yerself a tankard o' *rum*, and *let's hear yer boldest tale*! A final question for ye - _do ye wish t' introduce yourself t' the crew?_"}},{"type": "actions","elements": [{"type": "button","style": "primary","text": {"type": "plain_text","text": "announce yer presence","emoji": true},"value": "announce_presence"}]}]"#.to_string())
                .build(),
        ).await.unwrap();
    }

    async fn callback(&self, ctx: &Context<'_>) {
        // runs every 60 seconds
        //println!("callback");
        let utc = Utc::now().naive_utc();
        let time = London.from_utc_datetime(&utc);

        for cron in &self.crons {
            if cron.cron.matches(&time) {
                cron.channel
                    .post_message(ctx, cron.message.clone(), None)
                    .await
                    .unwrap();
            }
        }

        /*let remaining = Cron::parse_time("50 21 * * *", Some(Utc::now())).unwrap();
        println!("remaining {}", remaining);

        if remaining < (1000 * 60) {
            Channel {
                id: "C07KFUBDWN9".to_string(),
            }
            .post_message(ctx, Message::new().text("It's 10:50pm!".to_string()))
            .await
            .unwrap();
        }*/
    }

    /*async fn slash_command(
        &self,
        // TODO: thing
        ctx: InteractionContext,
        command: String,
        text: String,
        channel: Channel,
        user: User,
    ) -> InteractionContext {
        println!("got a command");
        match command.as_str() {
            "/pigeon" => pigeon(ctx, channel, user).await,
            _ => ctx,
        }
    }*/
}

/*#[slash_command]
async fn crow(ctx: InteractionContext<'_>, text: String, user: User, channel: Channel) {
    println!(
        "Recieved a command from <@{}> in <#{}> with text {}.",
        user.id, channel.id, text
    );

    channel
        .post_message(
            &ctx,
            Message::new().text("Hello, world!".to_string()).build(),
            true,
        )
        .await
        .unwrap();
}*/

/*fn crow() -> crate::client::interactions::SlashCommand {
    fn inner(
        ctx: InteractionContext<'_>,
        text: String,
        user: User,
        channel: Channel,
    ) -> BoxFuture<'_, ()> {
        async move {
            println!(
                "Recieved a command from <@{}> in <#{}> with text {}.",
                user.id, channel.id, text
            );
            channel
                .post_message(
                    &ctx,
                    Message::new().text("Hi!".to_string()).build(),
                    Some(user),
                )
                .await
                .unwrap();
        }
        .boxed()
    }
    SlashCommand {
        command: "crow".to_string(),
        execute: inner,
    }
}*/

#[slash_command]
async fn crow(ctx: InteractionContext<'_>, text: String, user: User, channel: Channel) {
    println!(
        "Recieved a command from <@{}> in <#{}> with text {}.",
        user.id, channel.id, text
    );

    channel
        .post_message(
            &ctx,
            Message::new().text("Hello, world!".to_string()).build(),
            Some(user),
        )
        .await
        .unwrap();
}

#[message_action]
async fn announce_presence(
    ctx: InteractionContext<'_>,
    user: User,
    name: String,
    display_name: String,
    channel: Channel,
) {
    Channel::new("C07DWPZ01K6".to_string())
        .post_message(
            &ctx,
            Message::new()
                .text(format!(
                    "arrr, everyone welcome <@{}> aboard to our crew!",
                    user.id
                ))
                .build(),
            None,
        )
        .await
        .unwrap();
}

/*async fn pigeon(mut ctx: InteractionContext, channel: Channel, user: User) -> InteractionContext {
    if user.id == "U078VGCHXSP" {
        ctx.respond(
            Message::new().text(
                "Welcome to pigeon! This is a work in progress, but modals are coming soon!"
                    .to_string(),
            ),
            true,
        );
        ctx.open_modal(PigeonModal);
    } else {
        ctx.respond(
            Message::new().text("Sorry, you can't use this command!".to_string()),
            true,
        );
    }

    ctx
}

struct PigeonModal;

impl Modal for PigeonModal {
    fn generate_view(&self, user: User) -> String {
        String::from_str(
            r#"{
    "type": "modal",
    "submit": {
        "type": "plain_text",
        "text": "Submit",
        "emoji": true
    },
    "close": {
        "type": "plain_text",
        "text": "Cancel",
        "emoji": true
    },
    "title": {
        "type": "plain_text",
        "text": "App menu",
        "emoji": true
    },
    "blocks": [
        {
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": "*Hi <fakelink.toUser.com|@David>!* Here's how I can help you:"
            }
        },
        {
            "type": "divider"
        },
        {
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": ":calendar: *Create event*\nCreate a new event"
            },
            "accessory": {
                "type": "button",
                "text": {
                    "type": "plain_text",
                    "text": "Create event",
                    "emoji": true
                },
                "style": "primary",
                "value": "click_me_123"
            }
        },
        {
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": ":clipboard: *List of events*\nChoose from different event lists"
            },
            "accessory": {
                "type": "static_select",
                "placeholder": {
                    "type": "plain_text",
                    "text": "Choose list",
                    "emoji": true
                },
                "options": [
                    {
                        "text": {
                            "type": "plain_text",
                            "text": "My events",
                            "emoji": true
                        },
                        "value": "value-0"
                    },
                    {
                        "text": {
                            "type": "plain_text",
                            "text": "All events",
                            "emoji": true
                        },
                        "value": "value-1"
                    },
                    {
                        "text": {
                            "type": "plain_text",
                            "text": "Event invites",
                            "emoji": true
                        },
                        "value": "value-1"
                    }
                ]
            }
        },
        {
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": ":gear: *Settings*\nManage your notifications and team settings"
            },
            "accessory": {
                "type": "static_select",
                "placeholder": {
                    "type": "plain_text",
                    "text": "Edit settings",
                    "emoji": true
                },
                "options": [
                    {
                        "text": {
                            "type": "plain_text",
                            "text": "Notifications",
                            "emoji": true
                        },
                        "value": "value-0"
                    },
                    {
                        "text": {
                            "type": "plain_text",
                            "text": "Team settings",
                            "emoji": true
                        },
                        "value": "value-1"
                    }
                ]
            }
        },
        {
            "type": "actions",
            "elements": [
                {
                    "type": "button",
                    "text": {
                        "type": "plain_text",
                        "text": "Send feedback",
                        "emoji": true
                    },
                    "value": "click_me_123"
                },
                {
                    "type": "button",
                    "text": {
                        "type": "plain_text",
                        "text": "FAQs",
                        "emoji": true
                    },
                    "value": "click_me_123"
                }
            ]
        }
    ]
}"#,
        )
        .unwrap()
    }

    fn handle_submit(&self) -> String {
        todo!()
    }

    fn handle_action(&self) {
        todo!()
    }
}*/

#[tokio::main]
async fn main() {
    let client = SlackClient::new()
        .port(3000)
        .slash_commands(vec![crow()])
        .message_actions(vec![announce_presence()])
        .event_handler(Handler {
            crons: vec![
                CronEvent::new(
                    "30 21 * * *",
                    "C07DWPZ01K6",
                    Message::from("_*YAAAWN*_ time for your daily recap <@U078VGCHXSP> :roo-yay:!"),
                ),
                CronEvent::new(
                    "0 7 * * 1-5",
                    "C07DWPZ01K6",
                    Message::from("_*YAAAAAWN*_ good morning everyone! :sleepybirb:"),
                ),
                CronEvent::new(
                    "15 10 * * 0,6",
                    "C07DWPZ01K6",
                    Message::from(
                        "_*CAW, CAW*_, good morning everybody! have a wonderful day :blahaj-heart:",
                    ),
                ),
            ],
        })
        .token(std::env::var("CROW_TOKEN").unwrap())
        .signing_secret(std::env::var("CROW_SECRET").unwrap())
        .build();

    client.run().await;
}
