use crate::TO_SERVER_CHANNEL;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Input, InputConfig, Level, Pull};
use esp_hal::peripherals::{GPIO0, GPIO18, GPIO39};
use log::info;
use nihility_edge_protocol::{KeyCode, KeyEvent, Message};

#[embassy_executor::task]
pub async fn button_task(
    key_enter: GPIO0<'static>,
    key_up: GPIO39<'static>,
    key_down: GPIO18<'static>,
) {
    let btn_enter = Input::new(key_enter, InputConfig::default().with_pull(Pull::Up));
    let btn_up = Input::new(key_up, InputConfig::default().with_pull(Pull::Up));
    let btn_down = Input::new(key_down, InputConfig::default().with_pull(Pull::Up));

    let mut prev_enter = btn_enter.level();
    let mut prev_up = btn_up.level();
    let mut prev_down = btn_down.level();

    let mut last_change_enter: u64 = 0;
    let mut last_change_up: u64 = 0;
    let mut last_change_down: u64 = 0;

    const DEBOUNCE_MS: u64 = 50;
    let to_server_sender = TO_SERVER_CHANNEL.sender();

    loop {
        let now = embassy_time::Instant::now().as_millis() as u64;

        let curr_enter = btn_enter.level();
        let curr_up = btn_up.level();
        let curr_down = btn_down.level();

        if curr_enter != prev_enter {
            if now.saturating_sub(last_change_enter) >= DEBOUNCE_MS {
                if curr_enter == Level::Low {
                    info!("Button Enter pressed");
                    to_server_sender
                        .send(Message::KeyEvent(KeyEvent {
                            key_code: KeyCode::Enter,
                            timestamp: now,
                        }))
                        .await;
                }
                prev_enter = curr_enter;
                last_change_enter = now;
            }
        }

        if curr_up != prev_up {
            if now.saturating_sub(last_change_up) >= DEBOUNCE_MS {
                if curr_up == Level::Low {
                    info!("Button Up pressed");
                    to_server_sender
                        .send(Message::KeyEvent(KeyEvent {
                            key_code: KeyCode::Up,
                            timestamp: now,
                        }))
                        .await;
                }
                prev_up = curr_up;
                last_change_up = now;
            }
        }

        if curr_down != prev_down {
            if now.saturating_sub(last_change_down) >= DEBOUNCE_MS {
                if curr_down == Level::Low {
                    info!("Button Down pressed");
                    to_server_sender
                        .send(Message::KeyEvent(KeyEvent {
                            key_code: KeyCode::Down,
                            timestamp: now,
                        }))
                        .await;
                }
                prev_down = curr_down;
                last_change_down = now;
            }
        }

        Timer::after(Duration::from_millis(20)).await;
    }
}
