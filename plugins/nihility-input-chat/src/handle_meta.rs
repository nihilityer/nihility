use onebot_v11::event::meta::Meta;
use tracing::info;

pub(super) async fn handle_meta(meta: Meta) {
    match meta {
        Meta::Lifecycle(lifecycle) => {
            info!("Lifecycle event: {:?}", lifecycle);
        }
        Meta::Heartbeat(heartbeat) => {
            info!("Heartbeat event: {:?}", heartbeat);
        }
    }
}
