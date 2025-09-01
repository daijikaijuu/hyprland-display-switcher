use std::process;

use anyhow::Result;
use hyprland::data::{Monitor, Monitors};
use hyprland::dispatch::{Dispatch, DispatchType};
use hyprland::shared::HyprData;
use iced::widget::{button, column, container, text};
use iced::{Element, Length, Task, Theme, alignment, event, Event};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::{Application, to_layer_message};

fn main() -> Result<(), iced_layershell::Error> {
    DisplaySwitcher::run(Settings {
        layer_settings: LayerShellSettings {
            size: Some((400, 0)),
            exclusive_zone: -1,
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            ..Default::default()
        },
        ..Default::default()
    })
}

struct DisplaySwitcher {
    state: State,
}

enum State {
    Loading,
    Loaded { monitors: Vec<Monitor> },
    Error { message: String },
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    LoadMonitors,
    MonitorsLoaded(Result<Vec<Monitor>, String>),
    SetMode(DisplayMode),
    Cancel,
    IcedEvent(Event),
}

#[derive(Debug, Clone)]
enum DisplayMode {
    Mirror,
    Extend,
    MainScreenOnly,
    SecondScreenOnly,
}

impl Application for DisplaySwitcher {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Task<Message>) {
        (Self::new(), Task::perform(
            async {
                Monitors::get()
                    .map(|monitors| monitors.into_iter().collect::<Vec<Monitor>>())
                    .map_err(|e| e.to_string())
            },
            Message::MonitorsLoaded,
        ))
    }

    fn namespace(&self) -> String {
        "display-switcher".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadMonitors => Task::perform(
                async {
                    Monitors::get()
                        .map(|monitors| monitors.into_iter().collect::<Vec<Monitor>>())
                        .map_err(|e| e.to_string())
                },
                Message::MonitorsLoaded,
            ),
            Message::MonitorsLoaded(Ok(monitors)) => {
                self.state = State::Loaded { monitors };
                Task::none()
            }
            Message::MonitorsLoaded(Err(err)) => {
                self.state = State::Error {
                    message: err.to_string(),
                };
                Task::none()
            }
            Message::SetMode(mode) => {
                if let State::Loaded { monitors } = &self.state {
                    if let Err(e) = apply_display_mode(&mode, monitors) {
                        eprintln!("Error applying display mode: {}", e);
                    }
                }
                process::exit(0);
            }
            Message::Cancel => {
                process::exit(0);
            }
            Message::IcedEvent(_event) => {
                // Handle iced events - this enables proper cursor handling
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.state {
            State::Loading => container(text("Loading display information...").size(30))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .into(),
            State::Error { message } => container(text(format!("Error {}", message)).size(20))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .into(),

            State::Loaded { monitors } => {
                let title = text("Display mode").size(24);
                let pc_screen_only = button("PC screen only")
                    .width(Length::Fill)
                    .style(button::primary)
                    .on_press(Message::SetMode(DisplayMode::MainScreenOnly));
                let duplicate = button("Duplicate")
                    .width(Length::Fill)
                    .style(button::primary)
                    .on_press(Message::SetMode(DisplayMode::Mirror));
                let extend = button("Extend")
                    .width(Length::Fill)
                    .style(button::primary)
                    .on_press(Message::SetMode(DisplayMode::Extend));
                let second_screen_only = button("Second screen only")
                    .width(Length::Fill)
                    .style(button::primary)
                    .on_press(Message::SetMode(DisplayMode::SecondScreenOnly));
                let cancel = button("Cancel")
                    .width(Length::Fill)
                    .style(button::secondary)
                    .on_press(Message::Cancel);

                let monitor_info = text(format!("Detected {} monitors", monitors.len())).size(16);

                container(
                    column![
                        title,
                        monitor_info,
                        pc_screen_only,
                        duplicate,
                        extend,
                        second_screen_only,
                        cancel
                    ]
                    .spacing(10)
                    .padding(20)
                    .width(Length::Fill),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .into()
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        event::listen().map(Message::IcedEvent)
    }

    fn style(&self, theme: &Self::Theme) -> iced_layershell::Appearance {
        iced_layershell::Appearance {
            background_color: iced::Color::TRANSPARENT,
            text_color: theme.palette().text,
        }
    }
}

impl DisplaySwitcher {
    fn new() -> Self {
        Self {
            state: State::Loading,
        }
    }
}

fn apply_display_mode(mode: &DisplayMode, monitors: &[Monitor]) -> Result<(), String> {
    if monitors.len() < 2 {
        return Ok(());
    }

    let primary = &monitors[0].name;
    let secondary = &monitors[1].name;

    match mode {
        DisplayMode::Mirror => {
            // First ensure both monitors are properly configured, then set mirror
            // Configure primary monitor
            Dispatch::call(DispatchType::Exec(&format!(
                "hyprctl keyword monitor \"{},1920x1080,0x0,1.0\"",
                primary
            )))
            .map_err(|e| e.to_string())?;

            // Configure secondary monitor to mirror primary
            Dispatch::call(DispatchType::Exec(&format!(
                "hyprctl keyword monitor \"{},1920x1080,0x0,1.0,mirror,{}\"",
                secondary, primary
            )))
            .map_err(|e| e.to_string())?;
        }
        DisplayMode::Extend => {
            // Configure monitors for extended display
            let _primary_mon = &monitors[0];
            let _secondary_mon = &monitors[1];

            // Use Full HD resolution for secondary monitor and calculate position
            let secondary_width = 1920;
            let secondary_height = 1080;
            let secondary_pos_x = secondary_width;

            // Configure primary monitor at 0,0 with auto resolution
            Dispatch::call(DispatchType::Exec(&format!(
                "hyprctl keyword monitor {},auto,0x0,auto",
                primary
            )))
            .map_err(|e| e.to_string())?;

            // Configure secondary monitor to the left with Full HD resolution and scale 1.0
            Dispatch::call(DispatchType::Exec(&format!(
                "hyprctl keyword monitor {},{}x{},{}x0,1.0",
                secondary, secondary_width, secondary_height, secondary_pos_x
            )))
            .map_err(|e| e.to_string())?;
        }
        DisplayMode::SecondScreenOnly => {
            // Disable primary monitor
            Dispatch::call(DispatchType::Exec(&format!(
                "hyprctl keyword monitor {} disable",
                primary
            )))
            .map_err(|e| e.to_string())?;

            // Enable secondary monitor
            Dispatch::call(DispatchType::Exec(&format!(
                "hyprctl keyword monitor {},0x0,auto",
                secondary
            )))
            .map_err(|e| e.to_string())?;
        }
        DisplayMode::MainScreenOnly => {
            // Enable primary monitor
            Dispatch::call(DispatchType::Exec(&format!(
                "hyprctl keyword monitor {},0x0,auto",
                primary
            )))
            .map_err(|e| e.to_string())?;

            // Disable secondary monitor
            Dispatch::call(DispatchType::Exec(&format!(
                "hyprctl keyword monitor {} disable",
                secondary
            )))
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}