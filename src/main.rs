use tokio_tungstenite::connect_async;

#[tokio::main]
async fn main() {
    let connect_addr = std::env::args()
        .nth(1)
        .expect("This program requires at least one arguement");

    let url = url::Url::parse(&connect_addr).unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    let (tx, rx) = flume::unbounded();

    let socket_handle = tokio::spawn(theta_server::run_connection(ws_stream, tx));
    let reader_handle = tokio::spawn(theta_server::run_reader(rx));

    let _ending = tokio::join!(socket_handle, reader_handle);
}
