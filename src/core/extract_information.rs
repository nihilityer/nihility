use anyhow::{Result, anyhow};
use nihility_common::model::get_chat_completion;
use rust_i18n::t;

pub async fn get_extract_information(inspiration: String) -> Result<String> {
    let system_prompt = format!(
        "{}\nOther Additional Information:",
        t!("prompt.extract_information"),
    );
    let precis = match get_chat_completion(system_prompt, inspiration)
        .await?
        .get("text")
    {
        None => return Err(anyhow!("Model Output Error")),
        Some(text) => text.to_string(),
    };
    Ok(precis)
}
