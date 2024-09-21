use std::{convert::Infallible, future::Future, net::SocketAddr, pin::Pin, sync::Arc};
use tokio::sync::{mpsc::Sender, Mutex, RwLock};

use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    server::conn::http1,
    service::{service_fn, Service},
    Request, Response,
};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use tokio::net::TcpListener;

use super::events::Events;

mod events;
mod verification;

/// A generic slack request
/// We only have the type as that is the only value we need to see where the data should go.
#[derive(Deserialize)]
struct SlackRequest {
    pub(super) r#type: String,
}

async fn handle_req(
    req: Request<hyper::body::Incoming>,
    secret: String,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let content = match verification::verify(req, secret).await {
        Ok(content) => content,
        Err(_) => return Ok(Response::new(Full::new(Bytes::from("Invalid request")))),
    };

    let ty = match serde_json::from_str::<SlackRequest>(content.as_str()) {
        Ok(SlackRequest { r#type }) => r#type,
        Err(_) => return Ok(Response::new(Full::new(Bytes::from("Invalid request")))),
    };

    println!("got request with type {}", ty);

    match ty.as_str() {
        "url_verification" => Ok(Response::new(Full::new(
            verification::url_verification(content).await,
        ))),
        "event_callback" => Ok(Response::new(Full::new(
            events::handle_event(content).await,
        ))),
        _ => Ok(Response::new(Full::new(Bytes::from("Invalid request")))),
    }
}

pub async fn listen(port: u16, signing_secret: String) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Listening on: {}", addr);

    let service = RequestHandler {
        secret: signing_secret.clone(),
    };

    while let Ok((stream, _)) = listener.accept().await {
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        let svc = service.clone();

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, svc)
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

#[derive(Clone)]
struct RequestHandler {
    secret: String,
}

impl Service<Request<Incoming>> for RequestHandler {
    type Response = Response<Full<Bytes>>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        Box::pin(handle_req(req, self.secret.clone()))
    }
}
