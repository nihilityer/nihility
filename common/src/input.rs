use crate::context::Context;

/// 信息输入实体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputEntity {
    /// 标明信息来源,可以通过此信息实现回复
    pub context: Context,
    /// 实际输入
    pub input: Input,
    /// 对助手的附加提示
    pub additional: String,
}

/// 输入的定义以及分类
/// 目前尝试将所有输入都转换为文本进行输入
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    /// 文本类型的信息输入
    Text(String),
}