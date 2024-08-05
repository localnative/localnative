#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::Application;
use localnative_iced::settings;
use localnative_iced::LocalNative;
use tracing_subscriber::layer::{Layer, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;

fn main() -> Result<(), iced::Error> {
    let level = if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    let layer = tracing_subscriber::fmt::layer()
        .without_time()
        .with_level(true);

    let targets = ["sqlx", "localnative"];
    let filter = tracing_subscriber::filter::filter_fn(move |metadata| {
        metadata.level() <= &level
            && targets
                .iter()
                .any(|&target| metadata.target().starts_with(target))
    });

    tracing_subscriber::registry()
        .with(layer.with_filter(filter))
        .init();

    LocalNative::run(settings())?;
    Ok(())
}
