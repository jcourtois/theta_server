use futures_util::StreamExt;
use theta_server::models;
use tokio_tungstenite::connect_async;

#[tokio::main]
async fn main() {
    let connect_addr = std::env::args()
        .nth(1)
        .expect("This program requires at least one arguement");

    let url = url::Url::parse(&connect_addr).unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (sink, stream) = ws_stream.split();

    let write_handle = tokio::spawn(async {
        let auth = models::StreamOne::auth("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        let subscribe = models::StreamOne::subscribe(vec!["T"]);
        auth.chain(subscribe).map(Ok).forward(sink).await
    });

    let read_handle = tokio::spawn(async move {
        tokio::pin!(stream);
        loop {
            if let Some(Ok(res)) = stream.next().await {
                let data = res.into_text().unwrap().to_string();
                println!("{data}");
            }
        }
    });

    let _ending = tokio::join!(write_handle, read_handle);
}
