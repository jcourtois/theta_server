use futures_util::{future, pin_mut, StreamExt};
use tokio_tungstenite::connect_async;

#[tokio::main]
async fn main() {
    let connect_addr = std::env::args()
        .nth(1)
        .expect("This program requires at least one arguement");

    let url = url::Url::parse(&connect_addr).unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (write, read) = ws_stream.split();

    let (data_to_ws, data_from_ws) = (
        theta_server::process_outbound_socket_messages(write),
        theta_server::process_inbound_socket_messages(read),
    );

    pin_mut!(data_to_ws, data_from_ws);
    future::select(data_to_ws, data_from_ws).await;
}
