use nihility::run;
use nihility_common::init_inspiration_sender;
use nihility_config::NihilityConfigPlugin;
use nihility_log::NihilityerLogPlugin;

#[tokio::main]
async fn main() {
    NihilityConfigPlugin::init("config").await.unwrap();
    NihilityerLogPlugin::init().await.unwrap();
    let input_receiver = init_inspiration_sender(20).await;

    #[cfg(feature = "api-model")]
    {
        use nihility_model_api::NihilityApiModel;
        tokio::spawn(NihilityApiModel::init());
    }

    #[cfg(feature = "chat-bot")]
    {
        use nihility_input_chat::NihilityChatInput;
        tokio::spawn(NihilityChatInput::init());
    }
    #[cfg(feature = "simple-memory")]
    {
        use nihility_memory_simple::NihilitySimpleMemory;
        tokio::spawn(NihilitySimpleMemory::init());
    }
    run(input_receiver).await.unwrap();
}
