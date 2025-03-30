use std::ops::Add;
use anyhow::Result;
use nihility_common::intention::Intention;
use nihility_common::model::get_chat_completion;
use rust_i18n::t;

pub async fn intention_recognition(think: String) -> Result<Intention> {
    let intention_value =
        get_chat_completion(build_prompt(), think).await?;
    Ok(serde_json::from_value::<Intention>(intention_value)?)
}

fn build_prompt() -> String {
    let mut prompt = String::new();
    prompt = prompt.add(t!("prompt.intention_recognition").as_ref());
    prompt = prompt.add(t!("prompt.recall_intention").as_ref());
    prompt = prompt.add(t!("prompt.memorize_intention").as_ref());
    prompt = prompt.add(t!("prompt.explore_intention").as_ref());
    prompt = prompt.add(t!("prompt.deliver_intention").as_ref());
    prompt = prompt.add(t!("prompt.execute_intention").as_ref());
    prompt
}
