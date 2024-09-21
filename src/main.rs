use client::{channel::Channel, user::User, Context, EventHandler, SlackClient};
mod client;

struct Handler;

impl EventHandler for Handler {
    async fn member_joined_channel(
        &self,
        ctx: Context<'_>,
        channel: Channel,
        user: User,
        inviter: User,
    ) {
        channel
            .post_text(&ctx, format!("Welcome to the channel, <@{}>!", user.id))
            .await;
    }

    fn callback(&self, ctx: &Context) {
        // runs every 60 seconds
    }
}

#[tokio::main]
async fn main() {
    let client = SlackClient::new()
        .port(65535)
        .event_handler(Handler)
        .token(std::env::var("PIGEON_TOKEN").unwrap())
        .signing_secret(std::env::var("PIGEON_SECRET").unwrap())
        .build();

    client.run().await;
}
