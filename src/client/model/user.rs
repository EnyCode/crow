use serde::{Serialize, Serializer};

use crate::client::{Channel, ContextSupplier, Message, SlackApiResponse, SlackError};

#[derive(Default, Debug, Clone)]
pub struct User {
    pub id: String,
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.id)
    }
}

impl User {
    pub async fn send_dm(
        &self,
        ctx: &impl ContextSupplier<'_>,
        mut msg: Message,
    ) -> Result<(), SlackError> {
        // technically not a channel but oh well
        msg.channel = Some(Channel {
            id: self.id.clone(),
        });

        println!("{}", serde_json::to_string_pretty(&msg).unwrap());

        let response: SlackApiResponse = ctx
            .get_context()
            .client
            .post("https://slack.com/api/chat.postMessage")
            // TODO: make into struct
            .json(&msg)
            .header(
                "Authorization",
                &("Bearer ".to_string() + &ctx.get_context().token),
            )
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        if !response.ok {
            return Err(SlackError::ApiError(response.error.unwrap()));
        }

        Ok(())
    }

    pub(crate) fn new(id: String) -> Self {
        Self { id }
    }
}
