use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};

/// A rectangle in physical pixels (virtual-desktop coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Top-left position closest to `pos` that keeps a `window`-sized rectangle
/// entirely inside `area`. A window larger than `area` is pinned to its
/// top-left corner.
pub fn fit_in_area(pos: (i32, i32), window: (i32, i32), area: Rect) -> (i32, i32) {
    let max_x = (area.x + area.width - window.0).max(area.x);
    let max_y = (area.y + area.height - window.1).max(area.y);
    (pos.0.clamp(area.x, max_x), pos.1.clamp(area.y, max_y))
}

/// Top-left position for a popup centred under `anchor` with a `gap`, or
/// above it when there is no room below, kept inside `area`.
pub fn place_below_or_above(anchor: Rect, popup: (i32, i32), area: Rect, gap: i32) -> (i32, i32) {
    let x = anchor.x + (anchor.width - popup.0) / 2;
    let below = anchor.y + anchor.height + gap;
    let above = anchor.y - gap - popup.1;
    let fits_below = below + popup.1 <= area.y + area.height;
    let y = if fits_below || above < area.y { below } else { above };
    fit_in_area((x, y), popup, area)
}

/// Work area (the monitor minus the taskbar) of the monitor containing the
/// given physical point, falling back to the primary monitor.
pub fn work_area_at(window: &WebviewWindow, x: i32, y: i32) -> Option<Rect> {
    let monitor = window
        .monitor_from_point(x as f64, y as f64)
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten())?;
    let work = monitor.work_area();
    Some(Rect {
        x: work.position.x,
        y: work.position.y,
        width: work.size.width as i32,
        height: work.size.height as i32,
    })
}

/// Moves `window` to the cursor, kept fully inside the work area of the
/// monitor the cursor is on. The window's last monitor is not used: with
/// several screens it is often not where the user is.
pub fn move_window_to_cursor(window: &WebviewWindow, cursor_x: i32, cursor_y: i32) {
    let size = window
        .outer_size()
        .unwrap_or(PhysicalSize { width: 0, height: 0 });
    let (x, y) = match work_area_at(window, cursor_x, cursor_y) {
        Some(area) => fit_in_area(
            (cursor_x, cursor_y),
            (size.width as i32, size.height as i32),
            area,
        ),
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
        assert_eq!(fit_in_area((500, 400), (160, 160), SCREEN), (500, 400));
    }

    #[test]
    fn stays_inside_the_right_and_bottom_edges() {
        // A 330px window at 200% scaling near the corner must not overflow.
        assert_eq!(fit_in_area((1910, 1030), (330, 330), SCREEN), (1590, 710));
    }

    #[test]
    fn works_on_a_monitor_left_of_the_primary() {
        let left = Rect {
            x: -2560,
            y: 0,
            width: 2560,
            height: 1400,
        };
        assert_eq!(fit_in_area((-2600, -10), (160, 160), left), (-2560, 0));
        assert_eq!(fit_in_area((-100, 700), (160, 160), left), (-160, 700));
    }

    const SHELF: Rect = Rect {
        x: 800,
        y: 300,
        width: 165,
        height: 175,
    };

    #[test]
    fn popup_opens_centred_below_the_shelf() {
        assert_eq!(
            place_below_or_above(SHELF, (450, 350), SCREEN, 5),
            (800 + (165 - 450) / 2, 300 + 175 + 5)
        );
    }

    #[test]
    fn popup_flips_above_a_shelf_near_the_bottom() {
        let low = Rect { y: 860, ..SHELF };
        assert_eq!(place_below_or_above(low, (450, 350), SCREEN, 5), (658, 860 - 5 - 350));
    }

    #[test]
    fn popup_stays_on_screen_next_to_a_shelf_at_the_left_edge() {
        let left = Rect { x: 0, ..SHELF };
        assert_eq!(place_below_or_above(left, (450, 350), SCREEN, 5).0, 0);
    }

    #[test]
    fn oversized_window_is_pinned_to_the_top_left() {
        let tiny = Rect {
            x: 100,
            y: 100,
            width: 50,
            height: 50,
        };
        assert_eq!(fit_in_area((120, 120), (160, 160), tiny), (100, 100));
    }
}
