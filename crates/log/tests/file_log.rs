use tracing::{debug, error, info, warn};

#[test]
pub fn test_log() {
    nihility_log::init().unwrap();
    debug!("This is debug");
    info!("This is info");
    warn!("This is warn");
    error!("This is error");
}