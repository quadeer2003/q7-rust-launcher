use std::fs;
use std::path::{Path, PathBuf};

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
    pub description: Option<String>,
}

// Removed unused DesktopEntry struct; parse by hand in parse_desktop_file

pub fn load_apps() -> Vec<DesktopApp> {
    #[cfg(windows)]
    {
        let mut out = Vec::new();
        // Typical Start Menu locations
        let mut roots: Vec<PathBuf> = Vec::new();
        if let Ok(programdata) = std::env::var("ProgramData") {
            roots.push(PathBuf::from(programdata).join("Microsoft/Windows/Start Menu/Programs"));
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            roots.push(PathBuf::from(appdata).join("Microsoft/Windows/Start Menu/Programs"));
        }
        // Recursively scan for .lnk files
        for root in roots {
            if !root.exists() { continue; }
            let mut stack = vec![root];
            while let Some(dir) = stack.pop() {
                if let Ok(rd) = fs::read_dir(&dir) {
                    for ent in rd.flatten() {
                        let p = ent.path();
                        if p.is_dir() { stack.push(p); continue; }
                        if p.extension().and_then(|e| e.to_str()).map(|s| s.eq_ignore_ascii_case("lnk")).unwrap_or(false) {
                            if let Some(app) = parse_windows_shortcut(&p) {
                                out.push(app);
                            }
                        }
                        // Allow .exe directly present
                        if p.extension().and_then(|e| e.to_str()).map(|s| s.eq_ignore_ascii_case("exe")).unwrap_or(false) {
                            let name = p.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_string();
                            out.push(DesktopApp{ name, exec: Some(p.to_string_lossy().into()), icon: None, path: p.clone(), resolved_icon_path: None, description: None });
                        }
                    }
                }
            }
        }
        out
    }
    #[cfg(not(windows))]
    {
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
}

fn parse_desktop_file(path: &Path) -> Option<DesktopApp> {
    let content = fs::read_to_string(path).ok()?;
    // very light parser: only lines in [Desktop Entry]
    let mut in_entry = false;
    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut comment: Option<String> = None;
    let mut generic_name: Option<String> = None;
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
        if let Some(rest) = line.strip_prefix("Comment=") { comment = Some(rest.to_string()); }
        if let Some(rest) = line.strip_prefix("GenericName=") { generic_name = Some(rest.to_string()); }
    }
    let name = name?;
    let description = comment.or(generic_name);
    Some(DesktopApp{ name, exec, icon, path: path.to_path_buf(), resolved_icon_path: None, description })
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

#[cfg(not(windows))]
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
    if let Ok(home) = std::env::var("HOME") {
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

#[cfg(windows)]
pub fn resolve_icon_path(_icon_field: &Option<String>) -> Option<PathBuf> { None }

#[cfg(windows)]
fn parse_windows_shortcut(path: &Path) -> Option<DesktopApp> {
    // Simplified .lnk parser - just return the shortcut name for now
    // Full COM-based parsing can be implemented later if needed
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("?").to_string();
    // Use the shortcut path itself as the exec for now
    let exec_path = path.to_string_lossy().to_string();
    Some(DesktopApp{ name, exec: Some(exec_path), icon: None, path: path.to_path_buf(), resolved_icon_path: None, description: None })
}
