mod config;
mod display;
mod state;
mod ui;
mod app;

use app::DisplaySwitcher;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};

fn main() -> Result<(), iced_layershell::Error> {
    DisplaySwitcher::run(Settings {
        layer_settings: LayerShellSettings {
            size: Some((500, 800)),
            exclusive_zone: 0,
            anchor: Anchor::empty(),
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            ..Default::default()
        },
        ..Default::default()
    })
}