use anyhow::Result;
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    pub name: String,
    pub desc: String,
    pub tags: Vec<String>,
    pub params: Value,
}

pub type BoxStream<T> = Pin<Box<dyn Stream<Item = Result<T>> + Send + 'static>>;

/// 子模块方法调用特征
#[async_trait::async_trait]
pub trait Callable {
    /// 不修改模块内部数据的方法调用
    async fn call(&self, func_name: &str, param: Value) -> Result<Value>;

    /// 修改模块内部数据的方法调用
    async fn call_mut(&mut self, func_name: &str, param: Value) -> Result<Value>;

    /// 流式方法调用
    async fn call_stream(
        &self,
        func_name: &str,
        param: Value,
    ) -> Result<BoxStream<Value>>;
}

/// 子模块特征
#[async_trait::async_trait]
pub trait Module: Callable {
    /// 获取模块简介
    fn description(&self) -> &str;
    /// 子模块支持调用的所有低权限方法列表
    fn no_perm_func(&self) -> Vec<FunctionMetadata>;
    /// 子模块支持调用的所有高权限方法列表
    fn perm_func(&mut self) -> Vec<FunctionMetadata>;
}
