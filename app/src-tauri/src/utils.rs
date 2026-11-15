use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};

/// A rectangle in physical pixels (virtual-desktop coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Top-left position that opens the window at the cursor, moved just enough
/// to keep the whole window inside `area`. A window larger than `area` is
/// pinned to its top-left corner.
pub fn fit_at_cursor(cursor: (i32, i32), window: (i32, i32), area: Rect) -> (i32, i32) {
    let max_x = (area.x + area.width - window.0).max(area.x);
    let max_y = (area.y + area.height - window.1).max(area.y);
    (cursor.0.clamp(area.x, max_x), cursor.1.clamp(area.y, max_y))
}

/// Moves `window` to the cursor, kept fully inside the work area (excluding
/// the taskbar) of the monitor the cursor is on. The window's last monitor
/// is not used: with several screens it is often not where the user is.
pub fn move_window_to_cursor(window: &WebviewWindow, cursor_x: i32, cursor_y: i32) {
    let monitor = window
        .monitor_from_point(cursor_x as f64, cursor_y as f64)
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    let size = window
        .outer_size()
        .unwrap_or(PhysicalSize { width: 0, height: 0 });

    let (x, y) = match monitor {
        Some(monitor) => {
            let work = monitor.work_area();
            let area = Rect {
                x: work.position.x,
                y: work.position.y,
                width: work.size.width as i32,
                height: work.size.height as i32,
            };
            fit_at_cursor(
                (cursor_x, cursor_y),
                (size.width as i32, size.height as i32),
                area,
            )
        }
        None => (cursor_x, cursor_y),
    };
    let _ = window.set_position(PhysicalPosition { x, y });
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 1920,
        height: 1040,
    };

    #[test]
    fn opens_at_the_cursor_when_it_fits() {
        assert_eq!(fit_at_cursor((500, 400), (160, 160), SCREEN), (500, 400));
    }

    #[test]
    fn stays_inside_the_right_and_bottom_edges() {
        // A 330px window at 200% scaling near the corner must not overflow.
        assert_eq!(fit_at_cursor((1910, 1030), (330, 330), SCREEN), (1590, 710));
    }

    #[test]
    fn works_on_a_monitor_left_of_the_primary() {
        let left = Rect {
            x: -2560,
            y: 0,
            width: 2560,
            height: 1400,
        };
        assert_eq!(fit_at_cursor((-2600, -10), (160, 160), left), (-2560, 0));
        assert_eq!(fit_at_cursor((-100, 700), (160, 160), left), (-160, 700));
    }

    #[test]
    fn oversized_window_is_pinned_to_the_top_left() {
        let tiny = Rect {
            x: 100,
            y: 100,
            width: 50,
            height: 50,
        };
        assert_eq!(fit_at_cursor((120, 120), (160, 160), tiny), (100, 100));
    }
}
