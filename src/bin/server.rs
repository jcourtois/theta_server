use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let db: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let _db = db.clone();

        tokio::spawn(async move {
            process(socket, _db).await;
        });
    }
}

async fn process(_socket: TcpStream, _db: Arc<Mutex<HashMap<String, String>>>) {
    ()
}
