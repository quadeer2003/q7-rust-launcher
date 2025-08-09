mod search;
mod commands;
mod apps;
mod config;

use eframe::{egui, NativeOptions};
use egui::{RichText, TextStyle};
use std::sync::{Arc, Mutex};
use std::process::{Command, Stdio};
use std::env;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Default, Clone)]
struct Entry {
    title: String,
    subtitle: String,
    action: Action,
}

#[derive(Clone)]
enum Action {
    LaunchApp(String),
    OpenFile(String),
    RunCmd(String),
    WebSearch(String),
    None,
}

impl Default for Action {
    fn default() -> Self { Action::None }
}

struct AppState {
    query: String,
    results: Vec<Entry>,
    all_apps: Vec<apps::DesktopApp>,
    config: config::Config,
    selected: usize,
    center_frames_remaining: u8,
    focused_once: bool,
    icon_textures: HashMap<String, egui::TextureHandle>,
    last_input: Instant,
    last_fd_query: String,
}

impl Default for AppState {
    fn default() -> Self {
    Self {
        query: String::new(),
        results: vec![],
        all_apps: vec![],
    config: config::Config::default(),
        selected: 0,
        center_frames_remaining: 3,
        focused_once: false,
        icon_textures: HashMap::new(),
        last_input: Instant::now(),
        last_fd_query: String::new(),
    }
    }
}

impl AppState {
    fn refresh_results(&mut self, include_files: bool) {
        let raw = self.query.as_str();
        let file_mode = raw.starts_with("f ");
        let q_apps = if file_mode { raw[2..].trim() } else { raw.trim() };
        let q_files = if file_mode { Some(q_apps) } else { None };
        let q = q_apps;
        self.results.clear();
        if q.is_empty() { return; }

        // Web search via configurable prefixes
        for eng in &self.config.search_engines {
            if q.starts_with(&eng.prefix) {
                let term = q[eng.prefix.len()..].trim();
                if !term.is_empty() {
                    let url = config::build_search_url(eng, term);
                    self.results.push(Entry{
                        title: format!("Search {} for: {}", eng.name, term),
                        subtitle: "Open in default browser".into(),
                        action: Action::WebSearch(url),
                    });
                }
            }
        }

    // App matches
    let app_matches = apps::fuzzy_match_apps(&self.all_apps, q);
        for a in app_matches.into_iter().take(5) {
            self.results.push(Entry{
                title: a.name.clone(),
                subtitle: a.description.clone().filter(|s| !s.is_empty()).or_else(|| a.exec.clone()).unwrap_or_default(),
                action: Action::LaunchApp(a.exec_unescaped()),
            });
        }

        // File search only when user explicitly types 'f '<query>
        if include_files {
            if let Some(qf) = q_files {
                if !qf.is_empty() {
                    for f in search::fd_search(qf, 10) {
                        self.results.push(Entry{
                            title: format!("Open file: {}", f.display()),
                            subtitle: f.to_string_lossy().into(),
                            action: Action::OpenFile(f.to_string_lossy().into()),
                        });
                    }
                }
            }
        }

    // Fallback: treat as shell command
    self.results.push(Entry{
            title: format!("Run command: {}", q),
            subtitle: "Execute in background".into(),
            action: Action::RunCmd(q.into()),
        });
    }
}

fn center_pos_from_xrandr(initial_size: egui::Vec2) -> Option<egui::Pos2> {
    // Only works on X11 with xrandr available
    let out = Command::new("xrandr").arg("--current").output().ok()?;
    if !out.status.success() { return None; }
    let s = String::from_utf8_lossy(&out.stdout);
    // Find the line with ' primary ' then extract WxH+X+Y
    let mut geom = None;
    for line in s.lines() {
        if line.contains(" primary ") {
            if let Some(caps) = regex::Regex::new(r"(\d+)x(\d+)\+(\d+)\+(\d+)").ok()?.captures(line) {
                let w: f32 = caps.get(1)?.as_str().parse::<u32>().ok()? as f32;
                let h: f32 = caps.get(2)?.as_str().parse::<u32>().ok()? as f32;
                let x: f32 = caps.get(3)?.as_str().parse::<u32>().ok()? as f32;
                let y: f32 = caps.get(4)?.as_str().parse::<u32>().ok()? as f32;
                let cx = x + w / 2.0 - initial_size.x / 2.0;
                let cy = y + h / 2.0 - initial_size.y / 2.0;
                geom = Some(egui::pos2(cx, cy));
                break;
            }
        }
    }
    geom
}

fn run_action(a: &Action) {
    match a {
        Action::LaunchApp(cmd) => {
            let mut c = Command::new("sh");
            c.arg("-lc").arg(cmd);
            if env::var("LANG").is_err() { c.env("LANG", "C.UTF-8"); }
            if env::var("LC_ALL").is_err() { c.env("LC_ALL", "C.UTF-8"); }
            let _ = c.stdout(Stdio::null()).stderr(Stdio::null()).spawn();
        }
        Action::OpenFile(path) => {
            let mut c = Command::new("xdg-open");
            c.arg(path);
            if env::var("LANG").is_err() { c.env("LANG", "C.UTF-8"); }
            if env::var("LC_ALL").is_err() { c.env("LC_ALL", "C.UTF-8"); }
            let _ = c.stdout(Stdio::null()).stderr(Stdio::null()).spawn();
        }
        Action::RunCmd(cmd) => {
            let _ = crate::commands::run_shell(cmd);
        }
    Action::WebSearch(url) => {
            let mut c = Command::new("xdg-open");
            c.arg(url);
            if env::var("LANG").is_err() { c.env("LANG", "C.UTF-8"); }
            if env::var("LC_ALL").is_err() { c.env("LC_ALL", "C.UTF-8"); }
            let _ = c.stdout(Stdio::null()).stderr(Stdio::null()).spawn();
        }
        Action::None => {}
    }
}

fn main() -> eframe::Result<()> {
    let mut state = AppState::default();
    state.all_apps = apps::load_apps();
    state.config = config::load_config();

    let state = Arc::new(Mutex::new(state));

    const INITIAL_SIZE: egui::Vec2 = egui::vec2(700.0, 420.0);
    const ICON_SIZE_PX: f32 = 48.0; // Icon size in result rows
    const RESULT_TITLE_FONT_SIZE: f32 = 22.0;   // px-like units in egui
    const RESULT_SUBTITLE_FONT_SIZE: f32 = 13.5;
    const ROW_ROUNDING: f32 = 6.0;
    const ROW_INNER_XPAD: f32 = 12.0;
    const ROW_INNER_YPAD: f32 = 8.0;

    let options = NativeOptions{
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_inner_size(INITIAL_SIZE)
            .with_always_on_top()
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_simple_native("q7 launcher", options, move |ctx, _frame| {
        let mut st = state.lock().unwrap();

        // Center the window for a few initial frames to account for WM sizing quirks (i3/X11)
        if st.center_frames_remaining > 0 {
            let pos = center_pos_from_xrandr(INITIAL_SIZE)
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

        egui::CentralPanel::default().frame(
            egui::Frame::none().fill(egui::Color32::BLACK)
        ).show(ctx, |ui| {
            ui.add_space(8.0);

            // Centered typing area
            let mut resp: Option<egui::Response> = None;
            ui.vertical_centered(|ui| {
                // Draw a light rounded background for the input area
                let bg = egui::Color32::from_gray(30);
                let (rect, _) = ui.allocate_exact_size(egui::vec2(540.0, 44.0), egui::Sense::hover());
                let rounding = egui::Rounding::same(8.0);
                ui.painter().rect_filled(rect, rounding, bg);

                // Place the text edit inside with some padding
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

            // Ensure the caret/focus is in the search box on first open
            if !st.focused_once {
                if let Some(r) = &resp { r.request_focus(); }
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
                st.focused_once = true;
            }

            if resp.as_ref().map(|r| r.changed()).unwrap_or(false) {
                st.last_input = Instant::now();
                let file_mode = st.query.starts_with("f ");
                st.refresh_results(file_mode);
                if file_mode {
                    st.last_fd_query = st.query.clone();
                }
            }

            // Keyboard navigation
            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
            let up = ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
            let down = ui.input(|i| i.key_pressed(egui::Key::ArrowDown));
            if up { if st.selected > 0 { st.selected -= 1; } }
            if down { if st.selected + 1 < st.results.len() { st.selected += 1; } }
            if enter {
                if let Some(sel) = st.results.get(st.selected) {
                    run_action(&sel.action);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }

            ui.add_space(8.0);
            let mut clicked_idx: Option<usize> = None;
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    let desired_width = 600.0;
                    let items: Vec<(usize, Entry)> = st.results.iter().cloned().enumerate().collect();
                    for (idx, e) in items.into_iter() {
                        let is_selected = idx == st.selected;
                        let row_selected_bg = egui::Color32::from_gray(60);
                        let inner = egui::Frame::none()
                            .fill(if is_selected { row_selected_bg } else { egui::Color32::TRANSPARENT })
                            .inner_margin(egui::Margin::symmetric(ROW_INNER_XPAD, ROW_INNER_YPAD))
                            .rounding(egui::Rounding::same(ROW_ROUNDING))
                            .show(ui, |ui| {
                                ui.set_width(desired_width);
                                ui.horizontal(|ui| {
                                    // Icon slot (load lazily for app entries)
                                    if let Action::LaunchApp(_) = e.action {
                                        if let Some(app) = st.all_apps.iter().find(|a| a.name == e.title) {
                                            if let Some(icon_path) = apps::resolve_icon_path(&app.icon) {
                                                let key = format!("{}@{}", icon_path.to_string_lossy(), ICON_SIZE_PX as i32);
                                                if !st.icon_textures.contains_key(&key) {
                                                    let ext = icon_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
                                                    let mut decoded: Option<image::DynamicImage> = None;
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
                                                    if decoded.is_none() {
                                                        if let Ok(img) = std::fs::read(&icon_path) {
                                                            if let Ok(dynimg) = image::load_from_memory(&img) {
                                                                decoded = Some(dynimg);
                                                            }
                                                        }
                                                    }
                                                    if let Some(dynimg) = decoded {
                                                        let rgba = dynimg.into_rgba8();
                                                        let size = [rgba.width() as usize, rgba.height() as usize];
                                                        let pixels = rgba.into_vec();
                                                        let tex = ui.ctx().load_texture(
                                                            key.clone(),
                                                            egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
                                                            egui::TextureOptions::LINEAR,
                                                        );
                                                        st.icon_textures.insert(key.clone(), tex);
                                                    }
                                                }
                                                if let Some(tex) = st.icon_textures.get(&key) {
                                                    let sz = egui::vec2(ICON_SIZE_PX, ICON_SIZE_PX);
                                                    ui.add(egui::Image::new(tex).fit_to_exact_size(sz));
                                                    ui.add_space(10.0);
                                                }
                                            }
                                        }
                                    }

                                    ui.vertical(|ui| {
                                        let title = if is_selected {
                                            RichText::new(&e.title).strong().underline().size(RESULT_TITLE_FONT_SIZE)
                                        } else {
                                            RichText::new(&e.title).strong().size(RESULT_TITLE_FONT_SIZE)
                                        };
                                        ui.label(title);
                                        ui.add_space(2.0);
                                        ui.label(egui::RichText::new(&e.subtitle).weak().size(RESULT_SUBTITLE_FONT_SIZE));
                                    });
                                });
                            });
                        if inner.response.clicked() {
                            clicked_idx = Some(idx);
                        }
                        ui.add_space(6.0);
                    }
                });
            });
            if let Some(idx) = clicked_idx {
                let action = st.results[idx].action.clone();
                st.selected = idx;
                run_action(&action);
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }

            // File mode ('f ') is immediate now; no debounce needed
        });
    })?;

    Ok(())
}
