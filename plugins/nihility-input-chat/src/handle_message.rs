use nihility_common::context::{ChatBotScene, Context, Scene};
use nihility_common::input::Input::Text;
use nihility_common::input::InputEntity;
use onebot_v11::MessageSegment;
use onebot_v11::event::message::Message;
use tracing::{debug, error, info};
use crate::{self_id, sender};

pub(super) async fn handle_message(message: Message) {
    // WIP 之后提取发送者信息拼接到文本输入中
    match message {
        Message::PrivateMessage(pm) => {
            debug!("Received private message: {:?}", pm);
            let chat_bot_scene = ChatBotScene {
                id: self_id().await.unwrap(),
                sender_info: pm.sender.user_id.unwrap(),
                group_id: None
            };
            build_text_input(chat_bot_scene, pm.message).await;
        }
        Message::GroupMessage(gm) => {
            debug!("Received group message: {:?}", gm);
            let chat_bot_scene = ChatBotScene {
                id: self_id().await.unwrap(),
                sender_info: gm.sender.user_id.unwrap(),
                group_id: Some(gm.group_id)
            };
            build_text_input(chat_bot_scene, gm.message).await;
        }
    }
}

async fn build_text_input(chat_bot_scene: ChatBotScene, messages: Vec<MessageSegment>) {
    let mut input_text = String::new();
    let mut additional = String::from(
        "这段消息会附加之前消息需要补充说明的信息,之前的消息中有许多使用`@0@`类似的序号标记,这会用于补充说明中提示位置:\n",
    );
    let mut index_flag = 0;
    for message in messages {
        match message {
            MessageSegment::Text { data } => {
                input_text.push_str(data.text.as_str());
            }
            MessageSegment::At { data } => {
                input_text.push_str(format!("`@{}@`", index_flag).as_str());
                if data.qq == *"all" {
                    additional.push_str(
                        format!("在序号{}处,信息发送者@了群聊中所有人\n", index_flag).as_str(),
                    );
                } else {
                    additional.push_str(format!("在序号{}处,信息发送者@了{},这是一个QQ号,如果需要更多信息,需要使用这个QQ来调用查询QQ号详细信息\n", index_flag, data.qq).as_str());
                }
                index_flag += 1;
            }
            other => {
                info!("暂时还不能处理这个消息: {:?}", other);
            }
        }
    }

    let input_entity = InputEntity {
        context: Context {
            scene: Scene::ChatBot(chat_bot_scene),
            topic: None,
        },
        input: Text(input_text),
        additional,
    };
    if let Err(e) = sender(input_entity).await {
        error!("Failed to send message: {}", e);
    }
}
