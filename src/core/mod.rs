use tracing::{error, info};
use crate::core::extract_information::get_extract_information;
use crate::core::intention_recognition::intention_recognition;
use crate::core::think::think;

pub mod extract_information;
pub mod think;
pub mod intention_recognition;

pub async fn core_heat_flow(inspiration: String) {
    let summary = match get_extract_information(inspiration).await {
        Ok(result) => result,
        Err(e) => {
            error!("Get Inspiration Summary error: {}", e);
            return;
        }
    };
    info!("Summary: {}", summary);
    let think = match think(summary).await {
        Ok(result) => result,
        Err(e) => {
            error!("Think Exception: {}", e);
            return;
        }
    };
    info!("Think: {}", think);
    let intention = match intention_recognition(think).await {
        Ok(result) => result,
        Err(e) => {
            error!("Intention Recognition Exception: {}", e);
            return;
        }
    };
    info!("Intention: {:?}", intention);
}