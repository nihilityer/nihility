use image::ImageReader;
use nihility_module::{Callable, Module};
use nihility_module_browser_control::func::open_page::OpenPageParam;
use nihility_module_browser_control::func::press_key::PressKeyParam;
use nihility_module_browser_control::func::screenshot::ScreenshotParam;
use nihility_module_browser_control::BrowserControl;
use std::io::Cursor;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

#[tokio::test]
async fn test_screenshot() {
    nihility_log::init().expect("log init failed");
    let mut browser_control = BrowserControl::init_from_file_config()
        .await
        .expect("init failed");
    info!("{:?}", browser_control.perm_func());
    let page_id: String = serde_json::from_value(
        browser_control
            .call_mut(
                "open_page",
                serde_json::to_value(OpenPageParam {
                    url: "http://127.0.0.1:8080/html/test?username=admin&password=123456"
                        .to_string(),
                })
                .expect("failed to build open_page param"),
            )
            .await
            .expect("call failed"),
    )
    .expect("open page result exception");
    info!(page_id=%page_id);
    sleep(Duration::from_secs(1)).await;
    browser_control
        .press_key(PressKeyParam {
            page_id: page_id.to_string(),
            key: "ArrowDown".to_string(),
        })
        .await
        .expect("press key failed");
    sleep(Duration::from_secs(1)).await;
    let image_data = browser_control
        .screenshot(ScreenshotParam {
            page_id,
            selector: Some("#app > div".to_string()),
        })
        .await
        .expect("screenshot failed");
    let image = ImageReader::new(Cursor::new(image_data))
        .with_guessed_format()
        .expect("guessed format error")
        .decode()
        .expect("decode failed");
    image.save("output/screenshot.png").expect("save failed");
}
