pub mod polygon {
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
            if secret.len() != 33 {
                panic!("secret should be 33 characters in length")
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
            let target = targets.join(",");
            Request {
                action: Action::Subscribe,
                params: target,
            }
        }

        pub fn as_message(&self) -> Message {
            Message::text(serde_json::to_string(self).unwrap())
        }
    }

    #[cfg(test)]
    mod actions {
        use super::*;

        #[test]
        fn serialize_auth_properly() {
            assert_eq!(
                serde_json::to_string(&Request::auth(&"x".repeat(33))).unwrap(),
                r#"{"action":"auth","params":"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}"#
            );
        }

        #[test]
        fn serialize_auth_into_message() {
            assert_eq!(
                Request::auth(&"x".repeat(33)).as_message(),
                Message::text(r#"{"action":"auth","params":"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}"#)
            );
        }

        #[test]
        fn serialize_subscribe_properly() {
            assert_eq!(
                serde_json::to_string(&Request::subscribe(std::vec!["T"])).unwrap(),
                r#"{"action":"subscribe","params":"T"}"#
            );
            assert_eq!(
                serde_json::to_string(&Request::subscribe(std::vec!["T", "F"])).unwrap(),
                r#"{"action":"subscribe","params":"T,F"}"#
            );
            assert_eq!(
                serde_json::to_string(&Request::subscribe(std::vec!["MMM", "T", "F"])).unwrap(),
                r#"{"action":"subscribe","params":"MMM,T,F"}"#
            );
        }

        #[test]
        fn serialize_subscribe_into_message() {
            assert_eq!(
                Request::subscribe(std::vec!["T", "F"]).as_message(),
                Message::text(r#"{"action":"subscribe","params":"T,F"}"#)
            );
        }

        #[test]
        #[should_panic(expected = "secret should be 33 characters in length")]
        fn auth_needs_a_secret_33_chars_in_length() {
            Request::auth("");
        }

        #[test]
        #[should_panic(expected = "need to subscribe to something!")]
        fn subscribe_needs_at_least_one_target() {
            Request::subscribe(std::vec![]);
        }
    }
}

fn main() {}
