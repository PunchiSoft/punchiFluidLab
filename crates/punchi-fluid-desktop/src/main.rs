// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Desktop entry point: window, menus and wiring.

mod app;
mod export_panel;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Punchi Fluid Lab")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0]),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "punchi-fluid-lab",
        options,
        Box::new(|context| Ok(Box::new(app::FluidLabApp::new(context)))),
    )
}
