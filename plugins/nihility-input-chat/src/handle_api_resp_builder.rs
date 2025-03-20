use onebot_v11::api::resp::ApiRespBuilder;
use tracing::warn;

pub(super) async fn handle_api_resp_builder(api_resp_builder: ApiRespBuilder) {
    warn!("ApiRespBuilder Event: {:?}", api_resp_builder);
}