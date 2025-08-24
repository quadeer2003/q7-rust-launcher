#![cfg_attr(windows, windows_subsystem = "windows")]

mod search;
mod commands;
mod apps;
mod config;
mod theme;
mod app_state;
mod actions;
mod ui;
mod utils;
mod autocomplete;

use eframe::{egui, NativeOptions};
use std::sync::{Arc, Mutex};
use app_state::AppState;

fn main() -> eframe::Result<()> {
    let mut state = AppState::default();
    state.all_apps = apps::load_apps();
    for (i, a) in state.all_apps.iter().enumerate() {
        state.app_by_name.insert(a.name.clone(), i);
    }
    state.config = config::load_config();
    if let Some(name) = state.config.current_theme.as_deref() {
        if let Some(p) = theme::ThemePalette::from_name(name) {
            state.theme = p;
        }
    }
    
    // Load autocomplete words if configured
    state.load_autocomplete_words();

    let state = Arc::new(Mutex::new(state));

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_inner_size(ui::INITIAL_SIZE)
            .with_always_on_top()
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_simple_native("q7 launcher", options, move |ctx, _frame| {
        ui::render_ui(ctx, &state);
    })?;

    Ok(())
}