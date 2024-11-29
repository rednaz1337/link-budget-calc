#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use egui::{Vec2, ViewportBuilder};
mod calc;
mod app;

fn main() {
    let viewport_builder = ViewportBuilder::default().with_inner_size(Vec2::new(420.0, 600.0));
    let native_options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "Link Budget Calculator",
        native_options,
        Box::new(|cc| app::LinkBudgetApp::new(cc)),
    )
        .unwrap();
}
