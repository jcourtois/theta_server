pub mod models;

use futures_util::{future, pin_mut, stream, Future, StreamExt};
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
    models::StreamOne::auth("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
        .map(Ok)
        .forward(tls_sink)
}

fn process_inbound_socket_messages(tls_source: TlsSocketSource) -> impl Future<Output = ()> {
    tls_source.for_each(|message| async {
        let data = message.unwrap().into_data();
        tokio::io::stdout().write_all(&data).await.unwrap();
    })
}
