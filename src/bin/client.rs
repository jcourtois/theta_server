use futures_util::{future, pin_mut, stream, Future, Stream, StreamExt};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{protocol::Message, Error},
    MaybeTlsStream, WebSocketStream,
};

type TlsSocketSource = stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type TlsSocketSink = stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

#[tokio::main]
async fn main() {
    let connect_addr = std::env::args()
        .nth(1)
        .expect("This program requires at least one arguement");

    let url = url::Url::parse(&connect_addr).unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (write, read) = ws_stream.split();

    let (data_to_ws, data_from_ws) = (
        process_outbound_socket_messages(write),
        process_inbound_socket_messages(read),
    );

    pin_mut!(data_to_ws, data_from_ws);
    future::select(data_to_ws, data_from_ws).await;
}

fn process_outbound_socket_messages(
    tls_sink: TlsSocketSink,
) -> impl Future<Output = Result<(), Error>> {
    let auth = Initialize::auth();
    let _subscribe = Initialize::subscribe();
    auth.map(Ok).forward(tls_sink)
}

fn process_inbound_socket_messages(tls_source: TlsSocketSource) -> impl Future<Output = ()> {
    tls_source.for_each(|message| async {
        let data = message.unwrap().into_data();
        tokio::io::stdout().write_all(&data).await.unwrap();
    })
}

struct Initialize {}

impl Initialize {
    fn auth() -> impl Stream<Item = Message> {
        one_message_with(r#"{"action":"auth","params":"my_secret"}"#)
    }

    fn subscribe() -> impl Stream<Item = Message> {
        one_message_with(r#"{"action":"subscribe","params":"Q.T"}"#)
    }
}

fn one_message_with(json: &str) -> impl Stream<Item = Message> {
    let msg = Message::text(json);
    stream::once(future::ready(msg))
}

#[cfg(test)]
mod initialize {
    use super::*;
    use futures_test::{assert_stream_done, assert_stream_next};

    #[tokio::test]
    async fn can_create_auth_messages() {
        let expected = Message::text(r#"{"action":"auth","params":"my_secret"}"#);
        let mut r = Initialize::auth();
        assert_stream_next!(r, expected);
        assert_stream_done!(r);
    }

    #[tokio::test]
    async fn can_create_subscribe_messages() {
        let expected = Message::text(r#"{"action":"subscribe","params":"Q.T"}"#);
        let mut r = Initialize::subscribe();
        assert_stream_next!(r, expected);
        assert_stream_done!(r);
    }
}
