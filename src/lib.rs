pub mod models;

use futures_util::{stream, Future, StreamExt};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_tungstenite::{
    tungstenite::{protocol::Message, Error},
    MaybeTlsStream, WebSocketStream,
};

type TlsSocketSource = stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type TlsSocketSink = stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

pub fn process_outbound_socket_messages(
    tls_sink: TlsSocketSink,
) -> impl Future<Output = Result<(), Error>> {
    models::StreamOne::auth("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
        .map(Ok)
        .forward(tls_sink)
}

pub fn process_inbound_socket_messages(tls_source: TlsSocketSource) -> impl Future<Output = ()> {
    tls_source.for_each(|message| async {
        let data = message.unwrap().into_data();
        tokio::io::stdout().write_all(&data).await.unwrap();
    })
}
