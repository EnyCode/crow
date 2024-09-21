use std::{net::SocketAddr, sync::OnceLock, time::Duration};

use channel::Channel;
use events::Events;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use reqwest::{Client, ClientBuilder};
use tokio::{
    net::TcpListener,
    sync::mpsc::{Receiver, Sender},
};
use user::User;

pub mod channel;
pub mod events;
pub mod listener;
pub mod user;

static EVENTS: OnceLock<Sender<Events>> = OnceLock::new();

pub trait EventHandler {
    async fn member_joined_channel(
        &self,
        ctx: Context,
        channel: Channel,
        user: User,
        inviter: User,
    );
    fn callback(&self, ctx: &Context);
}

pub struct SlackClient<E>
where
    E: EventHandler + Send,
{
    port: u16,
    event_handler: E,
    event_queue: (Sender<Events>, Receiver<Events>),
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
                tokio::time::sleep(Duration::from_secs(60)).await;

                self.event_queue.0.send(Events::Callback).await.unwrap();
            }
        });

        loop {
            let event = self.event_queue.1.recv().await.unwrap();
            match event {
                Events::MemberJoinedChannel(member_joined_channel_event) => {
                    self.event_handler
                        .member_joined_channel(
                            Context {
                                client: &self.client,
                                token: self.token.clone(),
                            },
                            Channel {
                                id: member_joined_channel_event.channel,
                            },
                            User {
                                id: member_joined_channel_event.user,
                            },
                            User {
                                id: member_joined_channel_event.inviter,
                            },
                        )
                        .await
                }
                Events::Callback => self.event_handler.callback(&Context {
                    client: &self.client,
                    token: self.token.clone(),
                }),
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

    pub fn build(self) -> SlackClient<E> {
        SlackClient {
            port: self.port.unwrap(),
            event_handler: self.event_handler.unwrap(),
            client: self.client,
            event_queue: tokio::sync::mpsc::channel(3),
            signing_secret: self.signing_secret.unwrap(),
            token: self.token.unwrap(),
        }
    }
}

/// Holds stuff needed to interact with the Slack API
pub struct Context<'a> {
    pub(super) client: &'a Client,
    pub(super) token: String,
}
