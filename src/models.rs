use futures_util::{future, stream, Stream};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Request {
    action: Action,
    params: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
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
            params: String::from("A.") + &targets.join(","),
        }
    }

    pub fn as_message(&self) -> Message {
        Message::text(serde_json::to_string(self).unwrap())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Response {
    ev: String,
    pub status: Status,
    message: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Connected,
    AuthFailed,
    AuthSuccess,
    Success,
}

impl Response {
    pub fn is_connected(&self) -> bool {
        self.status == Status::Connected
    }

    pub fn is_auth_success(&self) -> bool {
        self.status == Status::AuthSuccess
    }

    pub fn is_success(&self) -> bool {
        self.status == Status::Success
    }

    pub fn from_message(message: Message) -> Vec<Response> {
        let str = message.into_text().unwrap();
        serde_json::from_str::<Vec<Response>>(&str).unwrap()
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
    fn deserialize_connection_confirmation() {
        assert_eq!(
            serde_json::from_str::<Vec<Response>>(
                r#"[{"ev":"status","status":"connected","message":"Connected Successfully"}]"#,
            )
            .unwrap(),
            vec![Response {
                ev: String::from("status"),
                status: Status::Connected,
                message: String::from("Connected Successfully")
            }]
        );
    }

    #[test]
    fn deserialize_authentication_confirmation() {
        assert_eq!(
            serde_json::from_str::<Vec<Response>>(
                r#"[{"ev":"status","status":"auth_success","message":"authenticated"}]"#,
            )
            .unwrap(),
            vec![Response {
                ev: String::from("status"),
                status: Status::AuthSuccess,
                message: String::from("authenticated")
            }]
        );
    }

    #[test]
    fn serialize_subscribe_properly() {
        assert_eq!(
            serde_json::to_string(&Request::subscribe(std::vec!["T"])).unwrap(),
            r#"{"action":"subscribe","params":"A.T"}"#
        );
        assert_eq!(
            serde_json::to_string(&Request::subscribe(std::vec!["T", "F"])).unwrap(),
            r#"{"action":"subscribe","params":"A.T,F"}"#
        );
        assert_eq!(
            serde_json::to_string(&Request::subscribe(std::vec!["MMM", "T", "F"])).unwrap(),
            r#"{"action":"subscribe","params":"A.MMM,T,F"}"#
        );
    }

    #[test]
    fn serialize_subscribe_into_message() {
        assert_eq!(
            Request::subscribe(std::vec!["T", "F"]).as_message(),
            Message::text(r#"{"action":"subscribe","params":"A.T,F"}"#)
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
        let expected = Message::text(r#"{"action":"subscribe","params":"A.T"}"#);
        let mut r = StreamOne::subscribe(std::vec!["T"]);
        assert_stream_next!(r, expected);
        assert_stream_done!(r);
    }
}
