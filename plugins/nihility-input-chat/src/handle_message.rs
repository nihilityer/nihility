use std::ops::Add;
use crate::sender;
use nihility_common::inspiration::Inspiration;
use onebot_v11::event::message::Message;
use onebot_v11::MessageSegment;
use tracing::{error, info};
use tracing::log::debug;
use nihility_common::model::get_image_text;
use anyhow::Result;

static ANALYZE_IMAGE_PROMPT: &str = r#"
你将作为一个助手系统的图片识别模块,结合当前思考内容将获取的图片详细地解析并描述图片的内容;
描述内容尽可能简洁精准;
当前思考内容将由用户提供;

特别说明：你只能输出符合JSON格式的响应，且必须包含'text'字段。
Output Example:
{"text":"example text"}
"#;

static IMAGE_ANALYZE_PRE: &str = "\n以下是消息中各图片的解析,url表示图片原本的链接或数据,text表示图片的解析内容:\n";

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
    let mut image_analyze = IMAGE_ANALYZE_PRE.to_string();
    for msg in msg_list {
        if let MessageSegment::Image { data } = msg {
            if let Some(url) = data.url {
                debug!("Image URL: {}", url);
                match get_image_base64(url).await {
                    Ok(encoded_image) => {
                        match get_image_text(ANALYZE_IMAGE_PROMPT.to_string(), String::new(), encoded_image.clone()).await {
                            Ok(result) => {
                                if let Some(text) = result.get("text") {
                                    info!("\nurl:{}\ntext:{}", encoded_image, text);
                                    image_analyze = image_analyze.add(format!("url: {}\ntext:{}\n", encoded_image, text).as_str());
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
    let message_str = serde_json::to_string(&message).unwrap().add(image_analyze.as_str());
    debug!("handle_message: {}", message_str);
    sender(Inspiration::ChatApp(message_str)).await.unwrap();
}

async fn get_image_base64(url: String) -> Result<String> {
    // TODO 通过此方式获取图片 最好实现base64编码后上传 onebot_v11::api::payload::ApiPayload
    // if url.starts_with("http") {
    //     let bytes = reqwest::get(url)
    //         .await?
    //         .bytes()
    //         .await?;
    //     Ok(STANDARD.encode(&bytes))
    // } else { 
    //     Ok(url)
    // }
    Ok(url)
}
