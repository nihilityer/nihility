use onebot_v11::event::notice::Notice;
use tracing::warn;

pub(super) async fn handle_notice(notice: Notice) {
    warn!("Netice Event: {:?}", notice);
}
