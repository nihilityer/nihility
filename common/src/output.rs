use crate::context::Context;

/// 信息输出实体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputEntity {
    /// 标明回复针对的场景
    pub context: Context,
    /// 实际输出
    pub input: Output
}

/// 输出的定义以及分类
/// 目前尝试将所有输出都转换为文本进行输出
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    /// 文本类型的信息输出
    Text(String),
}