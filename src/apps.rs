use std::fs;
use std::path::{Path, PathBuf};
use std::env;

use crate::search::fuzzy_score;

#[derive(Clone, Debug, Default)]
pub struct DesktopApp {
    pub name: String,
    pub exec: Option<String>,
    #[allow(unused)]
    pub icon: Option<String>,
    #[allow(unused)]
    pub path: PathBuf,
    pub resolved_icon_path: Option<PathBuf>,
}

// Removed unused DesktopEntry struct; parse by hand in parse_desktop_file

pub fn load_apps() -> Vec<DesktopApp> {
    let mut out = vec![];
    let mut dirs: Vec<String> = vec![
        "/usr/share/applications".into(),
        "/usr/local/share/applications".into(),
    ];
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(format!("{}/.local/share/applications", home));
    }
    for d in dirs {
        let p = Path::new(&d);
        if !p.exists() { continue; }
        if let Ok(rd) = fs::read_dir(p) {
            for ent in rd.flatten() {
                let path = ent.path();
                if path.extension().and_then(|s| s.to_str()) != Some("desktop") { continue; }
                if let Some(mut app) = parse_desktop_file(&path) {
                    // Pre-resolve icon path once to avoid repeated scanning later
                    app.resolved_icon_path = resolve_icon_path(&app.icon);
                    out.push(app);
                }
            }
        }
    }
    out
}

fn parse_desktop_file(path: &Path) -> Option<DesktopApp> {
    let content = fs::read_to_string(path).ok()?;
    // very light parser: only lines in [Desktop Entry]
    let mut in_entry = false;
    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() { continue; }
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry { continue; }
        if let Some(rest) = line.strip_prefix("Name=") { name = Some(rest.to_string()); }
        if let Some(rest) = line.strip_prefix("Exec=") { exec = Some(rest.to_string()); }
        if let Some(rest) = line.strip_prefix("Icon=") { icon = Some(rest.to_string()); }
    }
    let name = name?;
    Some(DesktopApp{ name, exec, icon, path: path.to_path_buf(), resolved_icon_path: None })
}

impl DesktopApp {
    pub fn exec_unescaped(&self) -> String {
        // Remove field codes like %u, %U, %f etc.
        let cmd = self.exec.clone().unwrap_or_default();
        let cleaned = cmd.split_whitespace().filter(|t| !t.starts_with('%')).collect::<Vec<_>>().join(" ");
        cleaned
    }
}

pub fn fuzzy_match_apps(apps: &[DesktopApp], query: &str) -> Vec<DesktopApp> {
    let mut scored: Vec<(i64, &DesktopApp)> = apps
        .iter()
        .filter_map(|a| fuzzy_score(query, &a.name).map(|s| (s, a)))
        .collect();
    scored.sort_by(|a,b| b.0.cmp(&a.0));
    scored.into_iter().map(|(_,a)| a.clone()).collect()
}

pub fn resolve_icon_path(icon_field: &Option<String>) -> Option<PathBuf> {
    let name = icon_field.as_ref()?;
    // If it's an absolute/relative path, try directly
    let direct = PathBuf::from(name);
    if direct.exists() {
        return Some(direct);
    }

    // Build candidate filenames
    let has_ext = Path::new(name).extension().is_some();
    let mut files: Vec<String> = Vec::new();
    if has_ext {
        files.push(name.clone());
    } else {
        for ext in ["png", "jpg", "jpeg", "xpm", "bmp"].iter() {
            files.push(format!("{}.{}", name, ext));
        }
    }

    // Common bases
    let mut bases: Vec<PathBuf> = vec![
        PathBuf::from("/usr/share/pixmaps"),
        PathBuf::from("/usr/share/icons"),
        PathBuf::from("/usr/local/share/icons"),
    ];
    if let Ok(home) = env::var("HOME") {
        bases.push(PathBuf::from(format!("{}/.local/share/icons", home)));
    }

    // Try hicolor and common sizes/themes first
    let themes = ["hicolor", "Adwaita", "Papirus"];
    let sizes = [
        "256x256","128x128","96x96","64x64","48x48","32x32","24x24","16x16",
    ];

    for base in &bases {
        // pixmaps root
        if base.ends_with("pixmaps") {
            for f in &files {
                let p = base.join(f);
                if p.exists() { return Some(p); }
            }
        }

        // themed icons
        for theme in &themes {
            for size in &sizes {
                for f in &files {
                    let p = base.join(theme).join(size).join("apps").join(f);
                    if p.exists() { return Some(p); }
                }
            }
        }
    }

    None
}
