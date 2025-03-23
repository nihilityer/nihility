use nihility::config::NihilityConfig;
use nihility::log::Log;
use nihility::run;
use nihility_common::init_inspiration_sender;

#[tokio::main]
async fn main() {
    let config = NihilityConfig::init().unwrap();
    Log::init(&config.log).unwrap();
    let input_receiver = init_inspiration_sender(20).await;
    #[cfg(feature = "api-model")]
    {
        use nihility_model_api::NihilityApiModel;
        NihilityApiModel::init(&config.api_model).await.unwrap();
    }

    #[cfg(feature = "chat-bot")]
    {
        use nihility_input_chat::NihilityChatInput;
        NihilityChatInput::init(&config.chat_bot).await.unwrap();
    }
    #[cfg(feature = "simple-memory")]
    {
        use nihility_memory_simple::NihilitySimpleMemory;
        NihilitySimpleMemory::init().await.unwrap();
    }
    run(input_receiver).await.unwrap();
}
