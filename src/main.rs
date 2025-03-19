use nihility::log::{Log, LogConfig};
use nihility::run;

#[tokio::main]
async fn main() {
    Log::init(&vec![LogConfig::default()]).unwrap();
    run().await.unwrap();
}
