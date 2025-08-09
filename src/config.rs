use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub search_engines: Vec<SearchEngine>,
    #[serde(default)]
    pub current_theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEngine {
    pub name: String,
    pub prefix: String,
    pub url: String, // must contain %s placeholder
}

impl Default for SearchEngine {
    fn default() -> Self {
        SearchEngine { name: "DuckDuckGo".into(), prefix: "?".into(), url: "https://duckduckgo.com/?q=%s".into() }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            search_engines: vec![
                SearchEngine { name: "DuckDuckGo".into(), prefix: "?".into(), url: "https://duckduckgo.com/?q=%s".into() },
                SearchEngine { name: "Google".into(), prefix: "g ".into(), url: "https://www.google.com/search?q=%s".into() },
                SearchEngine { name: "YouTube".into(), prefix: "yt ".into(), url: "https://www.youtube.com/results?search_query=%s".into() },
                SearchEngine { name: "Wikipedia".into(), prefix: "w ".into(), url: "https://en.wikipedia.org/wiki/Special:Search?search=%s".into() },
                SearchEngine { name: "GitHub".into(), prefix: "gh ".into(), url: "https://github.com/search?q=%s".into() },
            ],
            current_theme: Some("Dracula".into()),
        }
    }
}

pub fn load_config() -> Config {
    // Try XDG config path: ~/.config/q7-launcher/config.json
    let bd = xdg::BaseDirectories::with_prefix("q7-launcher").ok();
    if let Some(bd) = &bd {
        if let Some(path) = bd.find_config_file("config.json") {
            if let Ok(s) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<Config>(&s) {
                    return cfg;
                }
            }
        }
    }
    // Fallback to project-relative assets/config.json if present
    if let Ok(s) = std::fs::read_to_string("assets/config.json") {
        if let Ok(cfg) = serde_json::from_str::<Config>(&s) {
            return cfg;
        }
    }
    // Default built-ins
    Config::default()
}

pub fn save_config(cfg: &Config) -> std::io::Result<()> {
    // Ensure XDG config directory exists and write there
    let bd = xdg::BaseDirectories::with_prefix("q7-launcher").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let path = bd.place_config_file("config.json")
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    std::fs::write(path, json)
}

pub fn build_search_url(engine: &SearchEngine, term: &str) -> String {
    let enc = urlencoding::encode(term);
    if engine.url.contains("%s") {
        engine.url.replace("%s", enc.as_ref())
    } else {
        format!("{}{}", engine.url, enc)
    }
}
