pub mod log;

use anyhow::Result;
use nihility_common::inspiration::Inspiration;
use nihility_common::model::get_chat_completion;
use tokio::sync::mpsc::Receiver;
use tracing::{error, info, warn};

static EXTRACT_INFORMATION_PROMPT: &str = r#"
你的任务是将从聊天机器人输入的JSON数据中提取关键信息，并结合当前讨论话题进行语义消歧和详细描述;
处理步骤:
1. 完整解析输入的JSON结构，识别所有可能的信息节点
2. 提取核心交互要素：{发送者信息, 消息环境, 消息内容, 元数据}
3. 结合当前讨论话题的上下文语义，对多义性内容进行确定当前上下文中的实际语义
4. 输出当前输入的详细描述

特别说明：你只能输出符合JSON格式的响应，且必须包含'text'字段。
Output Example:
{"text":"example text"}
"#;

static UPDATE_THINK: &str = r#"
你将作为一个助手的思考模块;
根据用户输入更新当前思考内容;
当前思考内容与输入有比较大区别时,判断一下哪个更值得进一步思考,并且更新思考内容;
更新后的思考内容以JSON格式返回,返回示例:{"text":"example text"}
例如:
当前思考:当前群聊中正在讨论关于下一顿吃什么的问题.
用户输入:有人在群聊中提问,怎么提高AI提示词的效果.
正确处理:更新当前思考内容,思考如何帮助群友达到想要达到的目标.

当前思考内容为:
"#;

static NEED_REMEMBER_PROMPT: &str = r#"
你是一个记忆判断助手，请严格按以下规则工作：
1. 当用户输入是明确的事实陈述或需要记录的事项时，返回 JSON {{"text": true}}
2. 其他所有情况（包括疑问句、不确定陈述、不需要记忆的内容）都返回 {{"text": false}}

特别说明：你只能输出符合JSON格式的响应，且必须包含'flag'字段。
Output Example:
{"flag":"example text"}
"#;

pub async fn run(mut input_receiver: Receiver<Inspiration>) -> Result<()> {
    info!("Starting core thread");
    while let Some(entity) = input_receiver.recv().await {
        info!("{:?}", entity);
        match entity {
            Inspiration::ChatApp(chat_inspiration) => {
                let system_prompt = format!(
                    "{}\nOther Additional Information:",
                    EXTRACT_INFORMATION_PROMPT,
                );
                let precis = match get_chat_completion(system_prompt, chat_inspiration)
                    .await?
                    .get("text")
                {
                    None => {
                        error!("Model Output Error");
                        continue;
                    }
                    Some(text) => text.to_string(),
                };
                info!("{}", precis);

                let update_system_prompt = format!("{}\n", UPDATE_THINK);
                match get_chat_completion(update_system_prompt, precis.to_string())
                    .await?
                    .get("text")
                {
                    None => {
                        error!("Model Output Error");
                        continue;
                    }
                    Some(think) => {
                        info!("Update think: {}", think);
                    }
                }

                let need_remember =
                    match get_chat_completion(NEED_REMEMBER_PROMPT.to_owned(), precis.clone())
                        .await?
                        .get("flag")
                    {
                        None => {
                            error!("Model Output Error");
                            continue;
                        }
                        Some(flag_value) => match flag_value.as_bool() {
                            None => {
                                error!("Model Output Error");
                                continue;
                            }
                            Some(flag) => flag,
                        },
                    };
                info!("Need Remember: {}", need_remember);
                // TODO
            }
            Inspiration::Memory(memory_inspiration) => {
                warn!("Received inspiration: {:?}", memory_inspiration);
            }
        }
    }
    Ok(())
}
