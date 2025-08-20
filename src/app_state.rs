use crate::{apps, config, theme::ThemePalette, actions::{Action, Entry}, search};
use std::collections::HashMap;
use std::time::Instant;
use eframe::egui::TextureHandle;

pub struct AppState {
    pub query: String,
    pub results: Vec<Entry>,
    pub all_apps: Vec<apps::DesktopApp>,
    pub app_by_name: HashMap<String, usize>,
    pub config: config::Config,
    pub selected: usize,
    pub center_frames_remaining: u8,
    pub focused_once: bool,
    pub icon_textures: HashMap<String, TextureHandle>,
    pub last_input: Instant,
    pub last_fd_query: String,
    pub theme: ThemePalette,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            query: String::new(),
            results: vec![],
            all_apps: vec![],
            app_by_name: HashMap::new(),
            config: config::Config::default(),
            selected: 0,
            center_frames_remaining: 6,
            focused_once: false,
            icon_textures: HashMap::new(),
            last_input: Instant::now(),
            last_fd_query: String::new(),
            theme: ThemePalette::dracula(),
        }
    }
}

impl AppState {
    pub fn refresh_results(&mut self, include_files: bool) {
        let raw = self.query.as_str();
        let file_mode = raw.starts_with("f ");
        let q_apps = if file_mode { raw[2..].trim() } else { raw.trim() };
        let q_files = if file_mode { Some(q_apps) } else { None };
        let q = q_apps;
        self.results.clear();
        if q.is_empty() { return; }

        // Theme picker: type "theme" to list themes
        if q.starts_with("theme") {
            for name in ThemePalette::names().iter() {
                self.results.push(Entry {
                    title: (*name).to_string(),
                    subtitle: "Apply color scheme".into(),
                    action: Action::ApplyTheme((*name).to_string()),
                });
            }
            return;
        }

        // Web search via configurable prefixes
        for eng in &self.config.search_engines {
            if q.starts_with(&eng.prefix) {
                let term = q[eng.prefix.len()..].trim();
                if !term.is_empty() {
                    let url = config::build_search_url(eng, term);
                    self.results.push(Entry {
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
            self.results.push(Entry {
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
                        self.results.push(Entry {
                            title: format!("Open file: {}", f.display()),
                            subtitle: f.to_string_lossy().into(),
                            action: Action::OpenFile(f.to_string_lossy().into()),
                        });
                    }
                }
            }
        }

        // Fallback: treat as shell command
        self.results.push(Entry {
            title: format!("Run command: {}", q),
            subtitle: "Execute in background".into(),
            action: Action::RunCmd(q.into()),
        });
    }
}