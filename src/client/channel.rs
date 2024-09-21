use super::Context;

pub struct Channel {
    pub id: String,
}

impl Channel {
    pub async fn post_text(&self, ctx: &Context<'_>, text: String) {
        let response = ctx
            .client
            .post("https://slack.com/api/chat.postMessage")
            .json(&serde_json::json!({
                "channel": self.id,
                "text": text,
            }))
            .header("Authorization", &("Bearer ".to_string() + &ctx.token))
            .send()
            .await
            .unwrap();
        println!("response {:#?}", response);
        println!("response body {:#?}", response.text().await.unwrap());
        println!("Posting text: {}", text);
    }
}
