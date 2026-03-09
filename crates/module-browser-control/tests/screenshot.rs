use image::ImageReader;
use nihility_module::{Callable, Module};
use nihility_module_browser_control::func::screenshot::ScreenshotParam;
use nihility_module_browser_control::BrowserControl;
use std::io::Cursor;
use tracing::info;

#[tokio::test]
async fn test_screenshot() {
    nihility_log::init().expect("log init failed");
    let mut browser_control = BrowserControl::init_from_file_config()
        .await
        .expect("init failed");
    info!("{:?}", browser_control.perm_func());
    let image_data: Vec<u8> = serde_json::from_value(
        browser_control
            .call_mut(
                "screenshot",
                serde_json::to_value(ScreenshotParam {
                    url: "http://127.0.0.1:8080/html/test".to_string(),
                })
                .expect("failed to build screenshot param"),
            )
            .await
            .expect("call failed"),
    )
    .expect("failed");
    let image = ImageReader::new(Cursor::new(image_data))
        .with_guessed_format()
        .expect("guessed format error")
        .decode()
        .expect("decode failed");
    image.save("output/screenshot.png").expect("save failed");
}
