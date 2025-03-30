use crate::sender;
use anyhow::Result;
use nihility_common::inspiration::Inspiration;
use nihility_common::model::get_image_text;
use onebot_v11::MessageSegment;
use onebot_v11::event::message::Message;
use rust_i18n::t;
use std::ops::Add;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::{guess_format, ImageFormat};
use tracing::log::debug;
use tracing::{error};


pub(super) async fn handle_message(message: Message) {
    let mut msg_list = Vec::<MessageSegment>::new();
    match &message {
        Message::PrivateMessage(pm) => {
            for msg in pm.message.iter() {
                msg_list.push(msg.clone());
            }
        }
        Message::GroupMessage(gm) => {
            for msg in gm.message.iter() {
                msg_list.push(msg.clone());
            }
        }
    }
    let mut image_analyze = String::new();
    for msg in msg_list {
        if let MessageSegment::Image { data } = msg {
            if let Some(url) = data.url {
                debug!("Image URL: {}", url);
                match get_image_base64(&url).await {
                    Ok(encoded_image) => {
                        match get_image_text(
                            t!("prompt.analyze_image_system").to_string(),
                            t!("prompt.analyze_image_user").to_string(),
                            encoded_image.clone(),
                        )
                        .await
                        {
                            Ok(result) => {
                                if let Some(text) = result.get("text") {
                                    debug!("\nencoded_image:{}\nanalyze result:{}", encoded_image, text);
                                    if image_analyze.is_empty() { 
                                        image_analyze = t!("prompt.message_image_chunk").to_string();
                                    }
                                    image_analyze = image_analyze.add(
                                        format!("url: {}\ntext:{}\n", url, text).as_str(),
                                    );
                                }
                            }
                            Err(e) => {
                                error!("analyze image failed: {}", e);
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        error!("encode image to base64 failed: {}", e);
                        return;
                    }
                }
            }
        }
    }
    let message_str = serde_json::to_string_pretty(&message)
        .unwrap()
        .add(image_analyze.as_str());
    debug!("handle_message: {}", message_str);
    sender(Inspiration::External(message_str)).await.unwrap();
}

async fn get_image_base64(url: &String) -> Result<String> {
    if url.starts_with("http") {
        debug!("start download image, url:{:?}", url);
        let bytes = reqwest::get(url)
            .await?
            .bytes()
            .await?;
        let prefix = match guess_format(&bytes)? {
            ImageFormat::Jpeg => "data:image/jpeg;base64,",
            ImageFormat::Png => "data:image/png;base64,",
            ImageFormat::Gif => "data:image/gif;base64,",
            ImageFormat::WebP => "data:image/webp;base64,",
            un_support => {
                debug!("unsupported image format: {:?}", un_support);
                "data:application/octet-stream;base64,"
            },
        };
        Ok(format!("{}{}", prefix, STANDARD.encode(&bytes)))
    } else {
        Ok(url.to_string())
    }
}
