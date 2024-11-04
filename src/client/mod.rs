use std::{
    collections::HashMap, future::Future, net::SocketAddr, pin::Pin, sync::OnceLock, time::Duration,
};

use events::Events;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use interactions::{ActionFn, CommandFn, InteractionContext, MessageAction, SlashCommand};
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use tokio::{
    net::TcpListener,
    sync::mpsc::{Receiver, Sender},
};

pub mod events;
pub mod model;
#[macro_use]
pub mod interactions;
pub mod listener;

pub use model::{channel::Channel, message::Message, user::User};

static EVENTS: OnceLock<Sender<Events>> = OnceLock::new();

#[derive(Debug)]
pub enum SlackError {
    InvalidInput(String),
    ApiError(String),
}

#[derive(Deserialize)]
pub(super) struct SlackApiResponse {
    ok: bool,
    error: Option<String>,
}

pub trait EventHandler {
    async fn member_joined_channel(
        &self,
        ctx: Context,
        channel: Channel,
        user: User,
        inviter: Option<User>,
    );
    async fn callback(&self, ctx: &Context);
}

pub struct SlackClient<E>
where
    E: EventHandler + Send,
{
    port: u16,
    event_handler: E,
    event_queue: (Sender<Events>, Receiver<Events>),
    slash_commands: HashMap<String, CommandFn>,
    message_actions: HashMap<String, ActionFn>,
    signing_secret: String,
    token: String,
    client: Client,
}

impl<E> SlackClient<E>
where
    E: EventHandler + Send,
{
    pub fn new() -> SlackClientBuilder<E> {
        let client = ClientBuilder::new()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION"),
            ))
            .build();

        SlackClientBuilder {
            port: None,
            event_handler: None,
            slash_commands: Vec::new(),
            message_actions: Vec::new(),
            client: client.unwrap(),
            signing_secret: None,
            token: None,
        }
    }

    pub async fn run(mut self) {
        println!("Running on port {}", &self.port);

        EVENTS.get_or_init(|| self.event_queue.0.clone());

        tokio::task::spawn(listener::listen(self.port, self.signing_secret.clone()));

        tokio::task::spawn(async move {
            loop {
                self.event_queue.0.send(Events::Callback).await.unwrap();
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });

        // event dispatcher
        loop {
            let event = self.event_queue.1.recv().await.unwrap();
            match event {
                Events::MemberJoinedChannel(event) => {
                    self.event_handler
                        .member_joined_channel(
                            Context {
                                client: &self.client,
                                token: self.token.clone(),
                            },
                            event.channel,
                            event.user,
                            event.inviter,
                        )
                        .await
                }
                Events::SlashCommandCalled(event) => {
                    let ctx = InteractionContext {
                        ctx: Context {
                            client: &self.client,
                            token: self.token.clone(),
                        },
                        trigger_id: event.trigger_id,
                    };

                    println!("{} {}", event.command, &event.command[1..]);

                    println!("{:#?}", self.slash_commands);

                    if let Some(cmd) = self.slash_commands.get(&event.command[1..]) {
                        cmd(ctx, event.text, event.user, event.channel).await;
                    } else {
                        println!("No command found for {}", event.command);
                    }
                }
                Events::BlockMessageInteraction(event) => {
                    let ctx: InteractionContext<'_> = InteractionContext {
                        ctx: Context {
                            client: &self.client,
                            token: self.token.clone(),
                        },
                        trigger_id: event.trigger_id,
                    };

                    println!("{}", event.action);

                    println!("{:#?}", self.message_actions);

                    if let Some(cmd) = self.message_actions.get(&event.action) {
                        cmd(
                            ctx,
                            event.user,
                            event.username,
                            event.display_name,
                            event.channel,
                        )
                        .await;
                    } else {
                        println!("No action handler found for {}", event.action);
                    }
                }
                Events::Callback => {
                    self.event_handler
                        .callback(&Context {
                            client: &self.client,
                            token: self.token.clone(),
                        })
                        .await
                }
            }
        }
    }
}

pub struct SlackClientBuilder<E>
where
    E: EventHandler,
{
    port: Option<u16>,
    event_handler: Option<E>,
    slash_commands: Vec<SlashCommand>,
    message_actions: Vec<MessageAction>,
    client: Client,
    signing_secret: Option<String>,
    token: Option<String>,
}

impl<E> SlackClientBuilder<E>
where
    E: EventHandler + Send,
{
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn event_handler(mut self, event_handler: E) -> Self {
        self.event_handler = Some(event_handler);
        self
    }

    pub fn signing_secret(mut self, signing_secret: String) -> Self {
        self.signing_secret = Some(signing_secret);
        self
    }

    pub fn token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn slash_commands(mut self, commands: Vec<SlashCommand>) -> Self {
        self.slash_commands.extend(commands);
        self
    }

    pub fn slash_command(mut self, command: SlashCommand) -> Self {
        self.slash_commands.push(command);
        self
    }

    pub fn message_actions(mut self, actions: Vec<MessageAction>) -> Self {
        self.message_actions.extend(actions);
        self
    }

    pub fn message_action(mut self, action: MessageAction) -> Self {
        self.message_actions.push(action);
        self
    }

    pub fn build(self) -> SlackClient<E> {
        SlackClient {
            port: self.port.unwrap(),
            event_handler: self.event_handler.unwrap(),
            slash_commands: self
                .slash_commands
                .iter()
                .fold(HashMap::new(), |mut map, cmd| {
                    map.insert(cmd.command.clone(), cmd.execute);
                    map
                }),
            message_actions: self
                .message_actions
                .iter()
                .fold(HashMap::new(), |mut map, cmd| {
                    map.insert(cmd.action.clone(), cmd.execute);
                    map
                }),
            client: self.client,
            event_queue: tokio::sync::mpsc::channel(3),
            signing_secret: self.signing_secret.unwrap(),
            token: self.token.unwrap(),
        }
    }
}

/// Holds stuff needed to interact with the Slack API
#[derive(Debug, Clone)]
pub struct Context<'a> {
    pub(super) client: &'a Client,
    pub(super) token: String,
}

impl<'a> ContextSupplier<'a> for Context<'a> {
    fn get_context(&self) -> &Context<'a> {
        self
    }
}

pub trait ContextSupplier<'a> {
    fn get_context(&self) -> &Context<'a>;
}
