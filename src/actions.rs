#[cfg(not(windows))]
use std::process::{Command, Stdio};
#[cfg(not(windows))]
use std::env;

#[derive(Default, Clone)]
pub struct Entry {
    pub title: String,
    pub subtitle: String,
    pub action: Action,
}

#[derive(Clone)]
pub enum Action {
    LaunchApp(String),
    OpenFile(String),
    RunCmd(String),
    WebSearch(String),
    ApplyTheme(String),
    None,
}

impl Default for Action {
    fn default() -> Self { Action::None }
}

pub fn run_action(a: &Action) {
    match a {
        Action::LaunchApp(cmd) => {
            #[cfg(windows)]
            {
                let _ = crate::commands::run_windows_command_hidden("cmd", &["/C", cmd]);
            }
            #[cfg(not(windows))]
            {
                let mut c = Command::new("sh");
                c.arg("-lc").arg(cmd);
                if env::var("LANG").is_err() { c.env("LANG", "C.UTF-8"); }
                if env::var("LC_ALL").is_err() { c.env("LC_ALL", "C.UTF-8"); }
                let _ = c.stdout(Stdio::null()).stderr(Stdio::null()).spawn();
            }
        }
        Action::OpenFile(path) => {
            #[cfg(windows)]
            {
                let _ = crate::commands::run_windows_command_hidden("cmd", &["/C", "start", "", path]);
            }
            #[cfg(not(windows))]
            {
                let mut c = Command::new("xdg-open");
                c.arg(path);
                if env::var("LANG").is_err() { c.env("LANG", "C.UTF-8"); }
                if env::var("LC_ALL").is_err() { c.env("LC_ALL", "C.UTF-8"); }
                let _ = c.stdout(Stdio::null()).stderr(Stdio::null()).spawn();
            }
        }
        Action::RunCmd(cmd) => {
            let _ = crate::commands::run_shell(cmd);
        }
        Action::WebSearch(url) => {
            #[cfg(windows)]
            {
                let _ = crate::commands::run_windows_command_hidden("cmd", &["/C", "start", "", url]);
            }
            #[cfg(not(windows))]
            {
                let mut c = Command::new("xdg-open");
                c.arg(url);
                if env::var("LANG").is_err() { c.env("LANG", "C.UTF-8"); }
                if env::var("LC_ALL").is_err() { c.env("LC_ALL", "C.UTF-8"); }
                let _ = c.stdout(Stdio::null()).stderr(Stdio::null()).spawn();
            }
        }
        Action::ApplyTheme(_) => {
            // no-op here; theme is applied in UI state
        }
        Action::None => {}
    }
}