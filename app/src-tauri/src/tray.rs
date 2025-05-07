use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, Runtime,
};
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let check_update_i =
        MenuItem::with_id(app, "check_update", "Check for Updates", true, None::<&str>)?;
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
                        Ok(updater) => match updater.check().await {
                            Ok(Some(update)) => {
                                let empty = String::new();
                                let body = update.body.as_ref().unwrap_or(&empty);
                                let _ = app_handle.dialog()
                                    .message(format!("Update to {} is available!\n\nRelease notes: {}", update.version, body))
                                    .title("Update Available")
                                    .kind(MessageDialogKind::Info)
                                    .blocking_show();
                                
                                // Since we can't get the user's choice directly, we'll proceed with the update
                                let _ = update.download_and_install(|_, _| {}, || {}).await;
                            }
                            Ok(None) => {
                                let _ = app_handle.dialog()
                                    .message("You're running the latest version!")
                                    .title("No Updates")
                                    .kind(MessageDialogKind::Info)
                                    .blocking_show();
                            }
                            Err(e) => {
                                let _ = app_handle.dialog()
                                    .message(format!("Failed to check for updates: {}", e))
                                    .title("Update Error")
                                    .kind(MessageDialogKind::Error)
                                    .blocking_show();
                            }
                        },
                        Err(e) => {
                            let _ = app_handle.dialog()
                                .message(format!("Failed to check for updates: {}", e))
                                .title("Update Error")
                                .kind(MessageDialogKind::Error)
                                .blocking_show();
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
