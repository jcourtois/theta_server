use futures_util::{future, stream, Stream};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Request {
    action: Action,
    params: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
enum Action {
    Auth,
    Subscribe,
}

impl Request {
    pub fn auth(secret: &str) -> Request {
        if secret.len() != 32 {
            panic!("secret should be 32 characters in length")
        }
        Request {
            action: Action::Auth,
            params: secret.to_string(),
        }
    }

    pub fn subscribe(targets: Vec<&str>) -> Request {
        if targets.is_empty() {
            panic!("need to subscribe to something!")
        }
        Request {
            action: Action::Subscribe,
            params: String::from("Q.") + &targets.join(","),
        }
    }

    pub fn as_message(&self) -> Message {
        Message::text(serde_json::to_string(self).unwrap())
    }
}

pub struct StreamOne {}

impl StreamOne {
    pub fn auth(secret: &str) -> impl Stream<Item = Message> {
        stream::once(future::ready(Request::auth(secret).as_message()))
    }
    pub fn subscribe(targets: Vec<&str>) -> impl Stream<Item = Message> {
        stream::once(future::ready(Request::subscribe(targets).as_message()))
    }
}

#[cfg(test)]
mod actions {
    use super::*;

    #[test]
    fn serialize_auth_properly() {
        assert_eq!(
            serde_json::to_string(&Request::auth(&"x".repeat(32))).unwrap(),
            r#"{"action":"auth","params":"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}"#
        );
    }

    #[test]
    fn serialize_auth_into_message() {
        assert_eq!(
            Request::auth(&"x".repeat(32)).as_message(),
            Message::text(r#"{"action":"auth","params":"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}"#)
        );
    }

    #[test]
    fn serialize_subscribe_properly() {
        assert_eq!(
            serde_json::to_string(&Request::subscribe(std::vec!["T"])).unwrap(),
            r#"{"action":"subscribe","params":"Q.T"}"#
        );
        assert_eq!(
            serde_json::to_string(&Request::subscribe(std::vec!["T", "F"])).unwrap(),
            r#"{"action":"subscribe","params":"Q.T,F"}"#
        );
        assert_eq!(
            serde_json::to_string(&Request::subscribe(std::vec!["MMM", "T", "F"])).unwrap(),
            r#"{"action":"subscribe","params":"Q.MMM,T,F"}"#
        );
    }

    #[test]
    fn serialize_subscribe_into_message() {
        assert_eq!(
            Request::subscribe(std::vec!["T", "F"]).as_message(),
            Message::text(r#"{"action":"subscribe","params":"Q.T,F"}"#)
        );
    }

    #[test]
    #[should_panic(expected = "secret should be 32 characters in length")]
    fn auth_needs_a_secret_32_chars_in_length() {
        Request::auth("");
    }

    #[test]
    #[should_panic(expected = "need to subscribe to something!")]
    fn subscribe_needs_at_least_one_target() {
        Request::subscribe(std::vec![]);
    }

    use futures_test::{assert_stream_done, assert_stream_next};

    #[tokio::test]
    async fn can_create_auth_messages() {
        let expected =
            Message::text(r#"{"action":"auth","params":"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}"#);
        let mut r = StreamOne::auth("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert_stream_next!(r, expected);
        assert_stream_done!(r);
    }

    #[tokio::test]
    async fn can_create_subscribe_messages() {
        let expected = Message::text(r#"{"action":"subscribe","params":"Q.T"}"#);
        let mut r = StreamOne::subscribe(std::vec!["T"]);
        assert_stream_next!(r, expected);
        assert_stream_done!(r);
    }
}
