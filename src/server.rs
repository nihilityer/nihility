#[tokio::main]
async fn main() {
    nihility_log::init().expect("Failed to initialize logger");

    nihility_util_init::init()
        .await
        .expect("Failed to initialize");

    let server_config = nihility_config::get_config("server").expect("Failed to get server config");
    nihility_server::start_server(server_config)
        .await
        .expect("Failed to start server");
}
