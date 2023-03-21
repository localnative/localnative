#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::Application;
use localnative_iced::settings;
use localnative_iced::LocalNative;

fn main() -> iced::Result {
    LocalNative::run(settings())
}
