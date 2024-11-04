use std::{default, str::FromStr};

use serde::{Serialize, Serializer};

use crate::client::{
    model::user::User, Context, ContextSupplier, Message, SlackApiResponse, SlackError,
};

#[derive(Default, Debug, Clone)]
pub struct Channel {
    pub id: String,
}

impl Serialize for Channel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.id)
    }
}

impl Channel {
    pub async fn post_message(
        &self,
        ctx: &impl ContextSupplier<'_>,
        mut msg: Message,
        ephemeral: Option<User>,
    ) -> Result<(), SlackError> {
        msg.channel = Some(Channel {
            id: self.id.clone(),
        });

        println!("{}", serde_json::to_string_pretty(&msg).unwrap());

        let is_ephemeral = ephemeral.is_some();
        if is_ephemeral {
            msg.receiver = ephemeral;
        }

        let response: SlackApiResponse = ctx
            .get_context()
            .client
            .post(if is_ephemeral {
                "https://slack.com/api/chat.postEphemeral"
            } else {
                "https://slack.com/api/chat.postMessage"
            })
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
