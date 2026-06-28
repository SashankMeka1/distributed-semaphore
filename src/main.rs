use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::{Arc, Mutex};

mod resp;
mod store;
mod commands;
mod expiry;

use resp::{RespParser, RespValue};
use store::Store;

#[tokio::main]
async fn main() {
    let store = Arc::new(Mutex::new(Store::new()));
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening on port 6379");

    tokio::spawn(expiry::run(Arc::clone(&store)));

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("New connection: {}", addr);
        let store = Arc::clone(&store);
        tokio::spawn(handle_connection(socket, store));
    }
}

async fn handle_connection(mut socket: TcpStream, store: Arc<Mutex<Store>>) {
    let mut parser = RespParser::new();
    let mut buf = [0u8; 4096];

    loop {
        let n = match socket.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        parser.feed(&buf[..n]);

        while let Some(value) = parser.parse() {
            let args = match extract_args(value) {
                Some(a) => a,
                None => {
                    let err = RespValue::Error("ERR invalid command format".to_string()).encode();
                    let _ = socket.write_all(&err).await;
                    continue;
                }
            };

            let response = {
                let mut store = store.lock().unwrap();
                commands::dispatch(args, &mut store)
            };

            if socket.write_all(&response.encode()).await.is_err() {
                break;
            }
        }
    }
}

fn extract_args(value: RespValue) -> Option<Vec<String>> {
    match value {
        RespValue::Array(Some(items)) => items
            .into_iter()
            .map(|item| match item {
                RespValue::BulkString(Some(s)) => Some(s),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}
