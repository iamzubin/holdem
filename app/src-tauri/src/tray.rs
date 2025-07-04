use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Runtime, WebviewUrl, WebviewWindowBuilder,
};
// use tauri_plugin_dialog::DialogExt;
// use tauri_plugin_updater::UpdaterExt;
// use tauri::Emitter;

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let check_update_i =
        MenuItem::with_id(app, "check_update", "Check for Updates", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&check_update_i, &quit_i])?;

    let _ = TrayIconBuilder::with_id("tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "check_update" => {
                // Open a dedicated updater window
                // First check if window already exists
                if let Some(existing_window) = app.get_webview_window("updater") {
                    let _ = existing_window.show();
                    let _ = existing_window.set_focus();
                } else {
                    // Create a new window for the updater
                    let _ = WebviewWindowBuilder::new(
                        app,
                        "updater",
                        WebviewUrl::App("/updater".into())
                    )
                    .title("Software Updates")
                    .inner_size(500.0, 400.0)
                    .decorations(false)
                    .center()
                    .build();
                }
                // The update check is now handled directly by the Updater component
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
