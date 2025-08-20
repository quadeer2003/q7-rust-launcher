#[cfg(not(windows))]
use std::process::Command;
use eframe::egui;

#[cfg(not(windows))]
pub fn center_pos_from_xrandr_points(initial_size_points: egui::Vec2, pixels_per_point: f32) -> Option<egui::Pos2> {
    let out = Command::new("xrandr").arg("--current").output().ok()?;
    if !out.status.success() { return None; }
    let s = String::from_utf8_lossy(&out.stdout);
    
    let desired_w_px = initial_size_points.x * pixels_per_point;
    let desired_h_px = initial_size_points.y * pixels_per_point;

    // Helper to compute clamped center within a monitor rect (in pixels)
    let compute = |w: f32, h: f32, x: f32, y: f32| -> egui::Pos2 {
        let cx_px = x + (w - desired_w_px).max(0.0) / 2.0;
        let cy_px = y + (h - desired_h_px).max(0.0) / 2.0;
        egui::pos2(cx_px / pixels_per_point, cy_px / pixels_per_point)
    };

    // Parse geometry string like "1920x1080+0+0" or "1920x1080+1920+0"
    fn parse_geometry(line: &str) -> Option<(u32, u32, u32, u32)> {
        // Find a pattern like "WIDTHxHEIGHT+X+Y"
        for part in line.split_whitespace() {
            if let Some(x_pos) = part.find('x') {
                let remaining = &part[x_pos + 1..];
                if let Some(plus1_pos) = remaining.find('+') {
                    let remaining2 = &remaining[plus1_pos + 1..];
                    if let Some(plus2_pos) = remaining2.find('+') {
                        let w_str = &part[..x_pos];
                        let h_str = &remaining[..plus1_pos];
                        let x_str = &remaining2[..plus2_pos];
                        let y_str = &remaining2[plus2_pos + 1..];
                        
                        // Handle cases where there might be extra text after coordinates
                        let y_clean = y_str.chars()
                            .take_while(|c| c.is_ascii_digit())
                            .collect::<String>();
                        
                        if let (Ok(w), Ok(h), Ok(x), Ok(y)) = (
                            w_str.parse::<u32>(),
                            h_str.parse::<u32>(),
                            x_str.parse::<u32>(),
                            y_clean.parse::<u32>()
                        ) {
                            return Some((w, h, x, y));
                        }
                    }
                }
            }
        }
        None
    }

    // Try primary monitor first
    for line in s.lines() {
        if line.contains(" connected") && line.contains(" primary ") {
            if let Some((w, h, x, y)) = parse_geometry(line) {
                return Some(compute(w as f32, h as f32, x as f32, y as f32));
            }
        }
    }
    
    // Fallback: first connected monitor with geometry
    for line in s.lines() {
        if line.contains(" connected") {
            if let Some((w, h, x, y)) = parse_geometry(line) {
                return Some(compute(w as f32, h as f32, x as f32, y as f32));
            }
        }
    }
    None
}

#[cfg(windows)]
pub fn center_window_windows(ctx: &egui::Context, window_size: egui::Vec2) {
    // Use Windows API to get accurate monitor info and DPI-aware positioning
    use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, GetCursorPos, MonitorFromPoint, GetMonitorInfoA, MONITORINFOEXA, MONITOR_DEFAULTTONEAREST, GetDC, ReleaseDC, GetDesktopWindow};
    use winapi::um::wingdi::{GetDeviceCaps, LOGPIXELSX};
    use winapi::shared::windef::POINT;
    use std::mem;

    unsafe {
        // Get DPI scaling factor
        let desktop_dc = GetDC(GetDesktopWindow());
        let dpi = GetDeviceCaps(desktop_dc, LOGPIXELSX) as f32;
        ReleaseDC(GetDesktopWindow(), desktop_dc);
        let dpi_scale = dpi / 96.0; // 96 DPI is the standard

        // Get cursor position to find the current monitor
        let mut cursor_pos = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut cursor_pos) == 0 {
            // Fallback to primary monitor if cursor position fails
            let screen_width = GetSystemMetrics(SM_CXSCREEN) as f32;
            let screen_height = GetSystemMetrics(SM_CYSCREEN) as f32;
            
            let pos = egui::pos2(
                (screen_width - (window_size.x * dpi_scale)) / 2.0,
                (screen_height - (window_size.y * dpi_scale)) / 2.0,
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(window_size));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
            return;
        }

        // Get monitor info for the monitor containing the cursor
        let monitor = MonitorFromPoint(cursor_pos, MONITOR_DEFAULTTONEAREST);
        let mut monitor_info: MONITORINFOEXA = mem::zeroed();
        monitor_info.cbSize = mem::size_of::<MONITORINFOEXA>() as u32;
        
        if GetMonitorInfoA(monitor, &mut monitor_info as *mut _ as *mut _) != 0 {
            // Use the work area (screen minus taskbar) with DPI adjustment
            let work_rect = monitor_info.rcWork;
            let work_width = (work_rect.right - work_rect.left) as f32;
            let work_height = (work_rect.bottom - work_rect.top) as f32;
            
            let scaled_window_width = window_size.x * dpi_scale;
            let scaled_window_height = window_size.y * dpi_scale;
            
            let pos = egui::pos2(
                work_rect.left as f32 + (work_width - scaled_window_width) / 2.0,
                work_rect.top as f32 + (work_height - scaled_window_height) / 2.0,
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(window_size));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
        } else {
            // Final fallback with DPI scaling
            let screen_width = GetSystemMetrics(SM_CXSCREEN) as f32;
            let screen_height = GetSystemMetrics(SM_CYSCREEN) as f32;
            
            let pos = egui::pos2(
                (screen_width - (window_size.x * dpi_scale)) / 2.0,
                (screen_height - (window_size.y * dpi_scale)) / 2.0,
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(window_size));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
        }
    }
}