use uuid::Uuid;

/// 场景上下文,由插件维护,但是每次传递信息时需要携带
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub scene: Scene,
    pub topic: Topic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topic {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scene {
    /// 聊天机器人,uuid表示内部场景标识
    ChatBot(Uuid)
}
