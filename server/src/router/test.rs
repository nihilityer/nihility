use axum::Json;
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct Notification {
    id: u32,
    title: String,
    content: String,
    time: DateTime<Utc>, // 使用标准时间类型，序列化为 ISO8601
}

pub(crate) async fn test() -> Json<Vec<Notification>> {
    let now = Utc::now();
    Json(vec![
        Notification {
            id: 1,
            title: "系统警报".into(),
            content: "固件更新可用 v2.4.0".into(),
            time: now - Duration::minutes(10),
        },
        Notification {
            id: 2,
            title: "用户登录".into(),
            content: "管理员从 192.168.1.5 登录".into(),
            time: now - Duration::hours(1),
        },
        Notification {
            id: 3,
            title: "传感器错误".into(),
            content: "温度传感器 #4 读数超出范围".into(),
            time: now - Duration::days(1),
        },
        Notification {
            id: 4,
            title: "备份完成".into(),
            content: "每日备份成功完成".into(),
            time: now - Duration::days(1),
        },
        Notification {
            id: 5,
            title: "安全警告".into(),
            content: "检测到来自 IP 10.0.0.5 的登录失败尝试".into(),
            time: now - Duration::days(2),
        },
        Notification {
            id: 6,
            title: "维护通知".into(),
            content: "计划维护窗口将于凌晨 02:00 开始".into(),
            time: now - Duration::days(3),
        },
        Notification {
            id: 7,
            title: "存储空间不足".into(),
            content: "磁盘使用率超过 90% 阈值".into(),
            time: now - Duration::weeks(1),
        },
        Notification {
            id: 8,
            title: "更新成功".into(),
            content: "模块 B 已成功更新至 v1.2".into(),
            time: now - Duration::weeks(1),
        },
    ])
}
