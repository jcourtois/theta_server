use crate::models::{Request, Response};
use flume::{Receiver, Sender};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub mod models;

pub async fn run_connection<S>(mut connection: WebSocketStream<S>, msg_tx: Sender<Message>)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    if let Some(message) = connection.next().await {
        let message = message.expect("issue connecting");
        let results = Response::from_message(message);
        assert!(results.iter().any(|r| r.is_connected()));
        println!("connected...");
    }

    connection
        .send(Request::auth("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").as_message())
        .await
        .expect("unable to send auth!");

    if let Some(message) = connection.next().await {
        let message = message.expect("issue receiving auth repsonse");
        let results = Response::from_message(message);
        assert!(results.iter().any(|r| r.is_auth_success()));
        println!("authenticated...");
    }

    connection
        .send(Request::subscribe(vec!["*"]).as_message())
        .await
        .expect("unable to send subscribe!");

    if let Some(message) = connection.next().await {
        let message = message.expect("issue receiving subscription repsonse");
        let results = Response::from_message(message);
        assert!(results.iter().any(|r| r.is_success()));
        println!("subscribed...");
    }

    while let Some(message) = connection.next().await {
        let message = message.expect("failed to get message");
        msg_tx.send(message).expect("failed to send results");
    }
}

pub async fn run_reader(rx: Receiver<Message>) {
    let mut rx_stream = rx.into_stream();
    while let Some(message) = rx_stream.next().await {
        println!("received message: <{message}>");
    }
}
