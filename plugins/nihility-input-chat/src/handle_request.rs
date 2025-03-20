use onebot_v11::event::request::Request;
use tracing::warn;

pub(super) async fn handle_request(request: Request) {
    warn!("Request Event: {:?}", request);
}