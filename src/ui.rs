use crate::{app_state::AppState, actions::{Action, run_action}, theme::ThemePalette, config, apps, utils};
use eframe::egui::{self, RichText, TextStyle};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub const INITIAL_SIZE: egui::Vec2 = egui::vec2(700.0, 420.0);
const ICON_SIZE_PX: f32 = 48.0;
const RESULT_TITLE_FONT_SIZE: f32 = 22.0;
const RESULT_SUBTITLE_FONT_SIZE: f32 = 13.5;
const ROW_ROUNDING: f32 = 6.0;
const ROW_INNER_XPAD: f32 = 12.0;
const ROW_INNER_YPAD: f32 = 8.0;

pub fn render_ui(ctx: &egui::Context, state: &Arc<Mutex<AppState>>) {
    let mut st = state.lock().unwrap();

    // Center the window for a few initial frames
    if st.center_frames_remaining > 0 {
        #[cfg(windows)]
        {
            utils::center_window_windows(ctx, INITIAL_SIZE);
            st.center_frames_remaining -= 1;
        }
        #[cfg(not(windows))]
        {
            let pos = utils::center_pos_from_xrandr_points(INITIAL_SIZE, ctx.pixels_per_point() as f32)
                .unwrap_or_else(|| {
                    let screen = ctx.screen_rect();
                    egui::pos2(
                        screen.center().x - INITIAL_SIZE.x / 2.0,
                        screen.center().y - INITIAL_SIZE.y / 2.0,
                    )
                });
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(INITIAL_SIZE));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
            st.center_frames_remaining -= 1;
        }
    }

    egui::CentralPanel::default().frame(
        egui::Frame::none().fill(st.theme.bg)
    ).show(ctx, |ui| {
        ui.add_space(8.0);

        // Render search input
        let resp = render_search_input(ui, &mut st);

        // Handle input changes
        if resp.as_ref().map(|r| r.changed()).unwrap_or(false) {
            st.last_input = Instant::now();
            let file_mode = st.query.starts_with("f ");
            st.refresh_results(file_mode);
            if st.selected >= st.results.len() { 
                st.selected = st.results.len().saturating_sub(1); 
            }
            if file_mode {
                st.last_fd_query = st.query.clone();
            }
        }

        // Handle keyboard input
        handle_keyboard_input(ui, ctx, &mut st);

        ui.add_space(8.0);

        // Render results
        render_results(ui, &mut st);
    });
}

fn render_search_input(ui: &mut egui::Ui, st: &mut AppState) -> Option<egui::Response> {
    let mut resp: Option<egui::Response> = None;
    ui.vertical_centered(|ui| {
        // Draw input background
        let bg = st.theme.input_bg;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(540.0, 44.0), egui::Sense::hover());
        let rounding = egui::Rounding::same(8.0);
        ui.painter().rect_filled(rect, rounding, bg);

        // Place the text edit inside with padding
        let mut child = ui.child_ui(rect.shrink2(egui::vec2(10.0, 6.0)), *ui.layout());
        let r = child.add_sized(
            [520.0, 32.0],
            egui::TextEdit::singleline(&mut st.query)
                .hint_text("Apps | f <name> | ?q / g q")
                .font(TextStyle::Heading)
                .frame(false)
        );
        resp = Some(r);
    });

    // Focus on first open
    if !st.focused_once {
        if let Some(r) = &resp { 
            r.request_focus(); 
        }
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
        st.focused_once = true;
    }

    resp
}

fn handle_keyboard_input(ui: &egui::Ui, ctx: &egui::Context, st: &mut AppState) {
    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
    let up = ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
    let down = ui.input(|i| i.key_pressed(egui::Key::ArrowDown));
    
    if up && st.selected > 0 { 
        st.selected -= 1; 
    }
    if down && st.selected + 1 < st.results.len() { 
        st.selected += 1; 
    }
    
    if enter {
        if let Some(action) = st.results.get(st.selected).map(|e| e.action.clone()) {
            match action {
                Action::ApplyTheme(name) => {
                    if let Some(p) = ThemePalette::from_name(&name) {
                        st.theme = p;
                        st.config.current_theme = Some(name);
                        let _ = config::save_config(&st.config);
                    }
                }
                other => {
                    run_action(&other);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }
}

fn render_results(ui: &mut egui::Ui, st: &mut AppState) {
    let mut clicked_idx: Option<usize> = None;
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            let desired_width = 600.0;
            let items: Vec<(usize, crate::actions::Entry)> = st.results.iter().cloned().enumerate().collect();
            let mut selected_row_rect: Option<egui::Rect> = None;
            
            for (idx, e) in items.into_iter() {
                let is_selected = idx == st.selected;
                let row_selected_bg = st.theme.selection_bg;
                
                let inner = egui::Frame::none()
                    .fill(if is_selected { row_selected_bg } else { egui::Color32::TRANSPARENT })
                    .inner_margin(egui::Margin::symmetric(ROW_INNER_XPAD, ROW_INNER_YPAD))
                    .rounding(egui::Rounding::same(ROW_ROUNDING))
                    .show(ui, |ui| {
                        ui.set_width(desired_width);
                        ui.horizontal(|ui| {
                            // Render icon for app entries
                            render_icon(ui, st, &e);
                            
                            // Render text content
                            ui.vertical(|ui| {
                                let title = if is_selected {
                                    RichText::new(&e.title)
                                        .color(st.theme.fg)
                                        .strong()
                                        .underline()
                                        .size(RESULT_TITLE_FONT_SIZE)
                                } else {
                                    RichText::new(&e.title)
                                        .color(st.theme.fg)
                                        .strong()
                                        .size(RESULT_TITLE_FONT_SIZE)
                                };
                                ui.label(title);
                                ui.add_space(2.0);
                                ui.label(
                                    RichText::new(&e.subtitle)
                                        .color(st.theme.muted)
                                        .size(RESULT_SUBTITLE_FONT_SIZE)
                                );
                            });
                        });
                    });
                
                if is_selected { 
                    selected_row_rect = Some(inner.response.rect); 
                }
                if inner.response.clicked() {
                    clicked_idx = Some(idx);
                }
                ui.add_space(6.0);
            }
            
            // Handle scrolling to selected item
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::ArrowDown)) {
                if let Some(rect) = selected_row_rect { 
                    ui.scroll_to_rect(rect, Some(egui::Align::Center)); 
                }
            }
        });
    });

    // Handle clicked items
    if let Some(idx) = clicked_idx {
        st.selected = idx;
        if let Some(action) = st.results.get(idx).map(|e| e.action.clone()) {
            match action {
                Action::ApplyTheme(name) => {
                    if let Some(p) = ThemePalette::from_name(&name) {
                        st.theme = p;
                        st.config.current_theme = Some(name);
                        let _ = config::save_config(&st.config);
                    }
                }
                other => {
                    run_action(&other);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }
}

fn render_icon(ui: &mut egui::Ui, st: &mut AppState, entry: &crate::actions::Entry) {
    if let Action::LaunchApp(_) = entry.action {
        if let Some(&idx) = st.app_by_name.get(&entry.title) {
            let app = &st.all_apps[idx];
            let icon_path_owned: Option<std::path::PathBuf> = match (&app.resolved_icon_path, &app.icon) {
                (Some(p), _) => Some(p.clone()),
                (None, icon_field) => apps::resolve_icon_path(icon_field),
            };
            
            if let Some(icon_path) = icon_path_owned.as_ref() {
                let key = format!("{}@{}", icon_path.to_string_lossy(), ICON_SIZE_PX as i32);
                
                if !st.icon_textures.contains_key(&key) {
                    load_icon_texture(ui, st, icon_path, &key);
                }
                
                if let Some(tex) = st.icon_textures.get(&key) {
                    let sz = egui::vec2(ICON_SIZE_PX, ICON_SIZE_PX);
                    ui.add(egui::Image::new(tex).fit_to_exact_size(sz));
                    ui.add_space(10.0);
                }
            }
        }
    }
}

fn load_icon_texture(ui: &egui::Ui, st: &mut AppState, icon_path: &std::path::Path, key: &str) {
    let ext = icon_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
    let mut decoded: Option<image::DynamicImage> = None;
    
    // Handle SVG files
    if ext == "svg" || ext == "svgz" {
        let mut cmd = std::process::Command::new("rsvg-convert");
        cmd.arg("-w").arg(format!("{}", ICON_SIZE_PX as i32))
           .arg("-h").arg(format!("{}", ICON_SIZE_PX as i32))
           .arg(&icon_path)
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::null());
        if std::env::var("LANG").is_err() { cmd.env("LANG", "C.UTF-8"); }
        if std::env::var("LC_ALL").is_err() { cmd.env("LC_ALL", "C.UTF-8"); }
        if let Ok(out) = cmd.output() {
            if out.status.success() {
                if let Ok(dynimg) = image::load_from_memory(&out.stdout) {
                    decoded = Some(dynimg);
                }
            }
        }
    }
    
    // Handle other image formats
    if decoded.is_none() {
        if let Ok(img) = std::fs::read(&icon_path) {
            if let Ok(dynimg) = image::load_from_memory(&img) {
                decoded = Some(dynimg);
            }
        }
    }
    
    // Create texture from decoded image
    if let Some(dynimg) = decoded {
        let rgba = dynimg.into_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba.into_vec();
        let tex = ui.ctx().load_texture(
            key.to_string(),
            egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
            egui::TextureOptions::LINEAR,
        );
        st.icon_textures.insert(key.to_string(), tex);
    }
}