#[tokio::test]
async fn test_init() {
    nihility_log::init().expect("Failed to initialize logger");
    nihility_util_init::init()
        .await
        .expect("Failed to initialize");
}
