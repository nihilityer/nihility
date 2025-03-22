use crate::sender;
use nihility_common::inspiration::Inspiration;
use onebot_v11::event::message::Message;
use tracing::info;

pub(super) async fn handle_message(message: Message) {
    info!("handle_message: {:?}", message);
    sender(Inspiration::ChatApp(
        serde_json::to_string_pretty(&message).unwrap(),
    ))
    .await
    .unwrap();
}
