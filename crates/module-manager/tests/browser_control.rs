//! ModuleManager 集成测试
//!
//! 测试 ModuleManager 与各个模块（特别是 browser-control）的集成功能。
//! 由于 Chromium 的限制，这些测试必须串行运行。
//!
//! 运行方式：
//! ```bash
//! cargo test -p nihility-module-manager --test browser_control -- --test-threads=1
//! ```

use image::ImageReader;
use nihility_module_manager::{EmbedModule, ModuleManager, ModuleType};
use serde_json::json;
use std::io::Cursor;
use std::sync::Once;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

/// 全局日志初始化标志，确保日志只初始化一次
static INIT: Once = Once::new();

/// 初始化日志系统（幂等操作）
fn init_log() {
    INIT.call_once(|| {
        nihility_log::init().ok();
    });
}

/// 测试通过ModuleManager调用browser-control模块的功能
#[tokio::test]
async fn test_module_manager_browser_control() {
    // 初始化日志系统
    init_log();

    // 初始化ModuleManager
    let module_manager = ModuleManager::init_from_file_config()
        .await
        .expect("init module manager failed");

    info!("ModuleManager initialized successfully");

    // 获取已加载的模块列表
    let loaded_modules = module_manager.loaded_modules();
    info!("Loaded modules: {:?}", loaded_modules);

    let browser_control_type = ModuleType::Embed(EmbedModule::BrowserControl);

    // 查询browser-control模块的功能列表
    let functions = module_manager
        .query_module_functions(&browser_control_type)
        .await
        .expect("query module functions failed");

    info!(
        "Browser control no_perm_func count: {}",
        functions.no_perm_func.len()
    );
    info!(
        "Browser control perm_func count: {}",
        functions.perm_func.len()
    );

    // 打印所有功能
    for func in &functions.no_perm_func {
        info!("No perm function: {} - {}", func.name, func.desc);
    }
    for func in &functions.perm_func {
        info!("Perm function: {} - {}", func.name, func.desc);
    }

    // 调用open_page功能打开网页
    let open_page_result = module_manager
        .call_mut(
            &browser_control_type,
            "open_page",
            json!({
                "url": "https://www.rust-lang.org/"
            }),
        )
        .await
        .expect("call open_page failed");

    let page_id: String = serde_json::from_value(open_page_result).expect("parse page_id failed");
    info!("Page opened with id: {}", page_id);

    // 等待页面加载
    sleep(Duration::from_secs(3)).await;

    // 调用screenshot功能截图
    let screenshot_result = module_manager
        .call(
            &browser_control_type,
            "screenshot",
            json!({
                "page_id": page_id,
                "selector": null
            }),
        )
        .await
        .expect("call screenshot failed");

    let image_data: Vec<u8> =
        serde_json::from_value(screenshot_result).expect("parse image data failed");
    info!("Screenshot taken, size: {} bytes", image_data.len());

    // 保存截图到文件
    let image = ImageReader::new(Cursor::new(&image_data))
        .with_guessed_format()
        .expect("guessed format error")
        .decode()
        .expect("decode failed");

    std::fs::create_dir_all("output").expect("create output dir failed");
    image
        .save("output/module_manager_screenshot.png")
        .expect("save failed");

    info!("Screenshot saved to output/module_manager_screenshot.png");
}

/// 测试查询所有模块的功能列表
#[tokio::test]
async fn test_query_all_functions() {
    init_log();

    let module_manager = ModuleManager::init_from_file_config()
        .await
        .expect("init module manager failed");

    // 查询所有模块的功能
    let all_functions = module_manager.query_functions().await;

    info!("Total modules loaded: {}", all_functions.len());

    for (module_type, functions) in all_functions {
        info!(
            "Module: {:?}, no_perm: {}, perm: {}",
            module_type,
            functions.no_perm_func.len(),
            functions.perm_func.len()
        );
    }
}

/// 测试模块不存在的错误处理
#[tokio::test]
async fn test_module_not_found() {
    init_log();

    let module_manager = ModuleManager::init_from_file_config()
        .await
        .expect("init module manager failed");

    // 尝试调用不存在的模块
    let wasm_module = ModuleType::Wasm("nonexistent".to_string());

    let result = module_manager
        .call(&wasm_module, "some_func", json!({}))
        .await;

    assert!(result.is_err());
    info!("Expected error: {:?}", result.err());
}

/// 测试功能不存在的错误处理
#[tokio::test]
async fn test_function_not_found() {
    init_log();

    let module_manager = ModuleManager::init_from_file_config()
        .await
        .expect("init module manager failed");

    let browser_control_type = ModuleType::Embed(EmbedModule::BrowserControl);

    // 尝试调用不存在的功能
    let result = module_manager
        .call(&browser_control_type, "nonexistent_func", json!({}))
        .await;

    assert!(result.is_err());
    info!("Expected error: {:?}", result.err());
}
