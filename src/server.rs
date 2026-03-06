#[tokio::main]
async fn main() {
    nihility_log::init().expect("Failed to initialize logger");
    nihility_server::start_server().await
}
