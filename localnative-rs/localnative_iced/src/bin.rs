#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::Application;
use ln_iced::settings;
use ln_iced::LocalNative;

fn main() -> iced::Result {
    LocalNative::run(settings())
}
