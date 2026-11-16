use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

fn show_shelf(app: &AppHandle) {
    let _ = crate::commands::window_ops::reveal_main_window(app);
    crate::analytics::track(app, "shelf_shown", vec![("via", "tray".into())]);
}

/// Tray menu items, kept so the page can relabel them in the user's language.
pub struct TrayItems(Vec<(&'static str, MenuItem<tauri::Wry>)>);

/// Sets the tray menu labels (keys: show, paste, settings, checkUpdates,
/// quit). The menu is built in Rust before any page loads, so it starts in
/// English and the shelf page sends the translations once i18n is ready.
#[tauri::command]
pub fn set_tray_labels(
    items: tauri::State<'_, TrayItems>,
    labels: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    for (key, item) in &items.0 {
        if let Some(text) = labels.get(*key).filter(|t| !t.trim().is_empty()) {
            item.set_text(text).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let show_i = MenuItem::with_id(app, "show", "Show shelf", true, None::<&str>)?;
    let paste_i = MenuItem::with_id(app, "paste", "Add clipboard to shelf", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let check_update_i =
        MenuItem::with_id(app, "check_update", "Check for updates", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit Holdem", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    app.manage(TrayItems(vec![
        ("show", show_i.clone()),
        ("paste", paste_i.clone()),
        ("settings", settings_i.clone()),
        ("checkUpdates", check_update_i.clone()),
        ("quit", quit_i.clone()),
    ]));
    let menu = Menu::with_items(
        app,
        &[&show_i, &paste_i, &settings_i, &check_update_i, &separator, &quit_i],
    )?;

    // The shelf has no taskbar button, so without the tray there is no way to
    // quit or check for updates: fail startup loudly instead of ignoring it.
    TrayIconBuilder::with_id("tray")
        .icon(
            app.default_window_icon()
                .cloned()
                .ok_or_else(|| tauri::Error::AssetNotFound("tray icon".into()))?,
        )
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Holdem")
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => show_shelf(app),
            // Show the shelf only when something was added.
            "paste" if crate::drop_target::paste_clipboard(app) => show_shelf(app),
            "settings" => {
                let _ = crate::commands::window_ops::open_settings_window(app.clone(), None);
            }
            "quit" => {
                app.exit(0);
            }
            "check_update" => {
                // The update check itself is handled by the Updater component.
                let _ = crate::commands::window_ops::open_updater_window(app);
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
                show_shelf(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
