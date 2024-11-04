use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use hex::ToHex;
use hmac::{Hmac, Mac};
use http_body_util::BodyExt;
use hyper::{
    body::{Bytes, Incoming},
    Request,
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

pub(super) async fn verify(req: Request<Incoming>, signing_secret: String) -> Result<String, ()> {
    type HmacSha256 = Hmac<Sha256>;

    let headers = req.headers();
    let timestamp = match headers.get("X-Slack-Request-Timestamp") {
        Some(timestamp) => timestamp.to_str().unwrap(),
        None => return Err(()),
    };

    // Check if the timestamp is older than 60 seconds
    if SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - timestamp.parse::<u64>().unwrap()
        > 60
    {
        println!("timestamp is older than 60 seconds");
        return Err(());
    }

    let timestamp = String::from_str(timestamp).unwrap();
    let slack_signature = String::from(headers.get("X-Slack-Signature").unwrap().to_str().unwrap());

    let content = String::from_utf8(req.collect().await.unwrap().to_bytes().to_vec()).unwrap();
    println!("timestamp: {:?}, content: {:?}", timestamp, content);

    let basestring = String::from_str("v0:").unwrap() + &timestamp + ":" + &content;

    let mut mac = HmacSha256::new_from_slice(signing_secret.as_bytes()).unwrap();
    println!("{:?}", basestring);
    mac.update(basestring.as_bytes());

    let sig_bytes = mac.finalize().into_bytes();
    let signature = sig_bytes.as_slice();

    let s = String::from("v0=") + &signature.encode_hex::<String>();

    if s == slack_signature {
        return Ok(content);
    }

    println!(
        "failed to verify - signature: {:?}, slack_signature: {:?}",
        signature.encode_hex::<String>(),
        slack_signature
    );

    Err(())
}

#[derive(Serialize, Deserialize)]
pub struct UrlVerification {
    pub challenge: String,
}

pub(super) async fn url_verification(content: String) -> Bytes {
    let challenge = match serde_json::from_str::<UrlVerification>(&content) {
        Ok(UrlVerification { challenge }) => challenge,
        Err(_) => return Bytes::from("Invalid request"),
    };

    Bytes::from(format!("{}", challenge))
}
