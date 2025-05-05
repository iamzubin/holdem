use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime, Emitter,
};
use tauri_plugin_updater::UpdaterExt;

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let check_update_i = MenuItem::with_id(app, "check_update", "Check for Updates", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&check_update_i, &quit_i])?;

    let _ = TrayIconBuilder::with_id("tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "check_update" => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    match app_handle.updater() {
                        Ok(updater) => {
                            match updater.check().await {
                                Ok(Some(update)) => {
                                    let _ = update.download_and_install(|_, _| {}, || {}).await;
                                }
                                Ok(None) => {
                                    let _ = app_handle.emit("update_available", false);
                                }
                                Err(e) => {
                                    let _ = app_handle.emit("update_error", e.to_string());
                                }
                            }
                        }
                        Err(e) => {
                            let _ = app_handle.emit("update_error", e.to_string());
                        }
                    }
                });
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app);

    Ok(())
}
