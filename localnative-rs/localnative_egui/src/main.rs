/*
    Local Native
    Copyright (C) 2018-2019  Yi Wang

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! egui/eframe desktop front-end for Local Native.
//!
//! This is a thin shell over `localnative_core`: it calls the synchronous
//! `db` query layer directly (the same path `localnative_iced` uses) and, for
//! peer sync, hands a JSON command to `localnative_core::run_sync` on a
//! background thread.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use app::LocalNativeApp;
use eframe::egui;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::layer::{Layer, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;

fn main() -> eframe::Result {
    init_tracing();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 720.0])
            .with_min_inner_size([640.0, 420.0])
            .with_title("Local Native"),
        ..Default::default()
    };

    eframe::run_native(
        "Local Native",
        native_options,
        Box::new(|cc| Ok(Box::new(LocalNativeApp::new(cc)))),
    )
}

/// Mirror the tracing setup used by `localnative_iced`: log only `rusqlite` and
/// `localnative` targets, at DEBUG in debug builds and INFO in release.
fn init_tracing() {
    let level = if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    let layer = tracing_subscriber::fmt::layer()
        .without_time()
        .with_level(true);

    let targets = ["rusqlite", "localnative"];
    let filter = filter_fn(move |metadata| {
        metadata.level() <= &level
            && targets
                .iter()
                .any(|&target| metadata.target().starts_with(target))
    });

    tracing_subscriber::registry()
        .with(layer.with_filter(filter))
        .init();
}
