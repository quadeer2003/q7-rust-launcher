use crate::{apps, config, theme::ThemePalette, actions::{Action, Entry}, search, autocomplete::AutocompleteEngine};
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
    pub autocomplete: AutocompleteEngine,
    pub autocomplete_mode: bool,
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
            autocomplete: AutocompleteEngine::new(),
            autocomplete_mode: false,
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

        // Autocomplete mode: Check if we should show autocomplete suggestions
        if self.autocomplete_mode && self.autocomplete.has_words() {
            let suggestions = self.autocomplete.get_suggestions(q, 10);
            for suggestion in suggestions {
                self.results.push(Entry {
                    title: suggestion.clone(),
                    subtitle: "Copy to clipboard".into(),
                    action: Action::CopyToClipboard(suggestion),
                });
            }
            // In autocomplete mode, don't show other results
            return;
        }

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

    pub fn load_autocomplete_words(&mut self) {
        if let Some(file_path) = &self.config.autocomplete_words_file {
            // Create default autocomplete file if it doesn't exist
            if !std::path::Path::new(file_path).exists() {
                self.create_default_autocomplete_file(file_path);
            }
            
            if let Err(e) = self.autocomplete.load_from_file(file_path) {
                eprintln!("Failed to load autocomplete words from {}: {}", file_path, e);
            }
        }
    }

    fn create_default_autocomplete_file(&self, file_path: &str) {
        let default_words = vec![
            "rust", "python", "javascript", "typescript", "programming", "development",
            "algorithm", "data", "structure", "function", "variable", "string", "integer",
            "boolean", "array", "vector", "hashmap", "dictionary", "class", "object",
            "method", "interface", "trait", "enum", "struct", "async", "await", "promise",
            "future", "thread", "process", "memory", "pointer", "reference", "borrow",
            "ownership", "lifetime", "compile", "runtime", "debug", "test", "benchmark",
            "performance", "optimization", "refactor", "module", "package", "crate",
            "library", "framework", "api", "rest", "http", "https", "json", "xml",
            "yaml", "toml", "database", "sql", "nosql", "mongodb", "postgresql",
            "mysql", "sqlite", "redis", "cache", "session", "authentication",
            "authorization", "jwt", "oauth", "security", "encryption", "hash", "salt",
            "password", "user", "admin", "client", "server", "backend", "frontend",
            "fullstack", "web", "mobile", "desktop", "game", "graphics", "ui", "ux",
            "design", "layout", "component", "widget", "button", "input", "form",
            "validation", "error", "exception", "log", "monitor", "deploy", "ci", "cd",
            "docker", "kubernetes", "aws", "azure", "gcp", "linux", "windows", "macos",
            "terminal", "shell", "bash", "zsh", "fish", "vim", "emacs", "vscode",
            "intellij", "git", "github", "gitlab", "commit", "push", "pull", "merge",
            "branch", "tag", "release", "version", "semantic", "major", "minor", "patch",
            "changelog"
        ];
        
        let content = default_words.join(", ");
        if let Err(e) = std::fs::write(file_path, content) {
            eprintln!("Failed to create default autocomplete file {}: {}", file_path, e);
        }
    }

    pub fn toggle_autocomplete_mode(&mut self) {
        self.autocomplete_mode = !self.autocomplete_mode;
        // Clear current results to refresh with new mode
        self.results.clear();
    }
}