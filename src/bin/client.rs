use futures_util::{future, pin_mut, stream, Future, Stream, StreamExt};
use json::object;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
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
    auth().map(Ok).forward(tls_sink)
}

fn auth() -> impl Stream<Item = Message> {
    let auth = object! { action: "auth", params: "my_secret" };
    stream::iter(vec![Message::text(auth.to_string())])
}

fn process_inbound_socket_messages(tls_source: TlsSocketSource) -> impl Future<Output = ()> {
    tls_source.for_each(|message| async {
        let data = message.unwrap().into_data();
        tokio::io::stdout().write_all(&data).await.unwrap();
    })
}

async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}

struct Initialize {}

impl Initialize {
    fn start() -> Vec<Message> {
        let subscription = object! { action: "subscribe", params: "Q.T" };
        vec![]
    }
    fn auth() -> impl Stream<Item = Message> {
        one_message_with(object! { action: "auth", params: "my_secret" })
    }

    fn subscribe() -> impl Stream<Item = Message> {
        one_message_with(object! { action: "subscribe", params: "Q.T" })
    }
}

fn one_message_with(json: json::JsonValue) -> impl Stream<Item = Message> {
    let msg = Message::text(json.dump());
    stream::once(future::ready(msg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = Initialize::start();
        assert_eq!(result, 4);
    }
}
