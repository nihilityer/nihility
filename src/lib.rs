pub mod config;
pub mod log;

use anyhow::Result;
use nihility_common::idea::{MemoryIdea};
use nihility_common::inspiration::Inspiration;
use nihility_common::sender_memory_idea;
use tokio::sync::mpsc::Receiver;
use tracing::{info, warn};

static EXTRACT_INFORMATION_PROMPT: &str = r#"{
  "任务描述": "从聊天机器人输入的JSON数据中提取结构化关键信息，并结合当前讨论话题进行语义消歧和详细描述",
  "处理步骤": [
    "1. 完整解析输入的JSON结构，识别所有可能的信息节点",
    "2. 提取核心交互要素：{发送者信息, 消息环境, 消息内容, 元数据}",
    "3. 结合当前讨论话题的上下文语义，对多义性内容进行优先级排序",
    "4. 输出结构化信息描述和原始数据的语义映射关系"
  ],
  "关键字段说明": {
    "发送者信息": {
      "必取字段": ["user_id", "nickname", "role"],
      "处理说明": "识别管理员/普通成员身份，记录用户唯一标识"
    },
    "消息环境": {
      "必取字段": ["group_id", "message_type", "message_id"],
      "处理说明": "区分私聊/群聊环境，保留消息追溯ID"
    },
    "消息内容": {
      "处理流程": [
        "解析message数组的多段式结构",
        "分离at指令对象和实际文本内容",
        "保留原始文本和结构化数据的对应关系"
      ],
      "异常处理": "当存在非文本内容(如图片/表情)时记录类型标识"
    },
    "时间信息": {
      "处理方式": "将UNIX时间戳转换为可读格式并保留原始值"
    }
  },
  "话题关联机制": {
    "执行策略": [
      "建立话题关键词词库：从当前讨论主题提取核心名词/动词",
      "实施语义相似度匹配：使用余弦相似度算法比对消息内容与话题关键词",
      "当检测到多个潜在语义时，优先选择与当前话题相关性最高的解释"
    ],
    "示例说明": {
      "当前话题": "用户对聊天机器人智能程度的讨论",
      "输入消息": "你有什么头猪吗",
      "处理过程": [
        "识别到'头猪'与常见表达'头绪'的发音相似性",
        "结合当前AI能力讨论的上下文",
        "将输入解释为'你有什么头绪吗'的语义错误表达"
      ]
    }
  },
  "输出示例": {
    "基础信息": {
      "环境类型": "群组消息",
      "群组ID": 111111111,
      "消息ID": 2222222222,
      "发送时间": "2025-03-21 15:52:37 (UTC+8)"
    },
    "参与者": {
      "发送者ID": 3333333333,
      "显示名称": "划水玩家",
      "群组角色": "管理员"
    },
    "消息内容": {
      "定向对象": "4444444444",
      "原始文本": "有人说你很像真人,你有什么头猪吗",
      "语义修正建议": [
        {
          "原词": "头猪",
          "建议替换": "头绪",
          "置信度": 0.87,
          "依据": "当前讨论主题为AI智能程度评估，'头绪'更符合对话逻辑"
        }
      ]
    },
    "话题关联度": 0.92,
    "处理备注": "检测到文本输入异常，已根据上下文语义进行合理化修正"
  }
}"#;

pub async fn run(mut input_receiver: Receiver<Inspiration>) -> Result<()> {
    info!("Starting core thread");
    while let Some(entity) = input_receiver.recv().await {
        info!("{:?}", entity);
        match entity {
            Inspiration::ChatApp(chat_inspiration) => {
                sender_memory_idea(MemoryIdea::Query(chat_inspiration)).await?;
            }
            Inspiration::Memory(memory_inspiration) => {
                warn!("Received inspiration: {:?}", memory_inspiration);
            }
        }
    }
    Ok(())
}
