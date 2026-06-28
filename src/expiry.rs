use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use crate::store::Store;

pub async fn run(store: Arc<Mutex<Store>>) {
    loop {
        sleep(Duration::from_secs(1)).await;
        store.lock().unwrap().evict_expired();
    }
}
