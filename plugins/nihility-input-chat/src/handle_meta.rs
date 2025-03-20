use onebot_v11::event::meta::Meta;
use serde_json::Value;
use tracing::{error, info, warn};

pub(super) async fn handle_meta(meta: Meta) {
    match meta {
        Meta::Lifecycle(lifecycle) => {
            info!("Lifecycle event: {:?}", lifecycle);
        }
        Meta::Heartbeat(heartbeat) => match heartbeat.status {
            Value::Object(object) => {
                if let Some(good) = object.get("good") {
                    if *good != Value::Bool(true) {
                        error!("Heartbeat Exception");
                    }
                }
            }
            other => {
                warn!("Other Heartbeat Status: {:?}", other);
            }
        }
    }
}
