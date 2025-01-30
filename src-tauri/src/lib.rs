pub mod cli;
pub mod compressor;
pub mod streaming;

use crate::streaming::get_devices;
use atomic_float::AtomicF32;
use clap::CommandFactory;
use clap::Parser;
use cli::{Cli, Commands};
use cpal::traits::DeviceTrait;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};

use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

pub static VOLUME: AtomicF32 = AtomicF32::new(-20.0); // 默认音量值为-20
pub const DEFAULT_ATTACK: f32 = 25.0;
pub const DEFAULT_RELEASE: f32 = 50.0;

#[tauri::command]
fn get_volume() -> f32 {
    VOLUME.load(Ordering::Relaxed)
}

#[tauri::command]
fn set_volume(value: f32) -> f32 {
    VOLUME.store(value, Ordering::Relaxed);
    value
}

#[tauri::command]
fn volume_changed(handle: &tauri::AppHandle) {
    handle
        .emit("volume-changed", VOLUME.load(Ordering::Relaxed))
        .unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cli = Cli::try_parse().unwrap();
    if let Some(command) = cli.command {
        match command {
            Commands::List => get_devices()
                .into_iter()
                .filter_map(|d| d.name().ok())
                .for_each(|d| println!("{}", d)),
        }
        return;
    } else if cli.input_device.is_none() && cli.output_device.is_none() {
        let mut cmd = Cli::command();
        cmd.print_help().expect("Failed to print help");
        return;
    }
    assert!(
        cli.input_device.is_some() && cli.output_device.is_some(),
        "input and output device is required"
    );

    let audio_volume_up_shortcut = Shortcut::new(None, Code::AudioVolumeUp);
    let audio_volume_down_shortcut = Shortcut::new(None, Code::AudioVolumeDown);

    set_volume(cli.threshold as f32);
    let devices = get_devices();
    let input_device = devices
        .iter()
        .find(|d| d.name().unwrap() == cli.input_device.as_ref().unwrap().trim())
        .unwrap_or_else(|| {
            panic!(
                "input device {} not found",
                cli.input_device.as_ref().unwrap()
            )
        });
    let output_device = devices
        .iter()
        .find(|d| d.name().unwrap().trim() == cli.output_device.as_ref().unwrap().trim())
        .unwrap_or_else(|| {
            panic!(
                "output device {} not found",
                cli.output_device.as_ref().unwrap()
            )
        });
    let _res = streaming::create_stream(input_device, output_device, cli.threshold as f32);

    tauri::Builder::default()
        .plugin(
            // 监听音量 + - 快捷键
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    if shortcut == &audio_volume_up_shortcut {
                        VOLUME.fetch_add(1.0, Ordering::Relaxed);
                    } else if shortcut == &audio_volume_down_shortcut {
                        VOLUME.fetch_sub(1.0, Ordering::Relaxed);
                    }
                    // 发送音量变化事件，前端更新
                    volume_changed(app);
                })
                .build(),
        )
        .setup(move |app| {
            let window = app.get_webview_window("main").unwrap();
            // 设置窗口位置（因为未知原因没法将窗口设在每次点击托盘的鼠标位置）
            window
                .set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
                    1400, 900,
                )))
                .unwrap();

            let last_click_time = Mutex::new(Instant::now() - Duration::from_secs(1));
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .show_menu_on_left_click(false)
                .on_tray_icon_event(move |_app_handle, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } => {
                        // 防抖
                        let mut last_click_time = last_click_time.lock().unwrap();
                        if last_click_time.elapsed() < Duration::from_millis(100) {
                            return;
                        }
                        *last_click_time = Instant::now();

                        if !window.is_visible().unwrap() {
                            _ = window.show();
                        } else {
                            _ = window.hide();
                        }
                    }
                    TrayIconEvent::Click {
                        button: MouseButton::Right,
                        ..
                    } => {
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .build(app)?;
            app.global_shortcut().register(audio_volume_up_shortcut)?;
            app.global_shortcut().register(audio_volume_down_shortcut)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_volume, set_volume,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
