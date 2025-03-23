use crate::sender;
use nihility_common::inspiration::Inspiration;
use onebot_v11::event::message::Message;
use tracing::info;

pub(super) async fn handle_message(message: Message) {
    let message_str = serde_json::to_string_pretty(&message).unwrap();
    info!("handle_message: {}", message_str);
    sender(Inspiration::ChatApp(message_str)).await.unwrap();
}
