use std::process;

use anyhow::Result;
use hyprland::data::{Monitor, Monitors};
use hyprland::dispatch::{Dispatch, DispatchType};
use hyprland::shared::HyprData;
use iced::widget::{button, column, container, text};
use iced::{Element, Length, Task, Theme, alignment};

fn main() -> iced::Result {
    iced::application(
        "Display Switcher",
        DisplaySwitcher::update,
        DisplaySwitcher::view,
    )
    .subscription(DisplaySwitcher::subscription)
    .theme(|_| Theme::Dark)
    .run()
}

struct DisplaySwitcher {
    state: State,
}

enum State {
    Loading,
    Loaded { monitors: Vec<Monitor> },
    Error { message: String },
}

#[derive(Debug, Clone)]
enum Message {
    LoadMonitors,
    MonitorsLoaded(Result<Vec<Monitor>, String>),
    SetMode(DisplayMode),
    Cancel,
}

#[derive(Debug, Clone)]
enum DisplayMode {
    Mirror,
    Extend,
    MainScreenOnly,
    SecondScreenOnly,
}

impl DisplaySwitcher {
    fn new() -> Self {
        Self {
            state: State::Loading,
        }
    }

    fn update(state: &mut Self, message: Message) -> Task<Message> {
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
                state.state = State::Loaded { monitors };
                Task::none()
            }
            Message::MonitorsLoaded(Err(err)) => {
                state.state = State::Error {
                    message: err.to_string(),
                };
                Task::none()
            }
            Message::SetMode(mode) => {
                if let State::Loaded { monitors } = &state.state
                    && let Err(e) = apply_display_mode(&mode, monitors)
                {
                    eprintln!("Error applying display mode: {}", e);
                }
                // Exit after applying the mode
                process::exit(0);
            }
            Message::Cancel => {
                // Exit the application
                process::exit(0);
            }
        }
    }

    fn subscription(_state: &Self) -> iced::Subscription<Message> {
        iced::event::listen().map(|_| Message::LoadMonitors)
    }

    #[allow(mismatched_lifetime_syntaxes)]
    fn view(state: &Self) -> Element<Message> {
        match &state.state {
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
                    .on_press(Message::SetMode(DisplayMode::MainScreenOnly));
                let duplicate = button("Duplicate")
                    .width(Length::Fill)
                    .on_press(Message::SetMode(DisplayMode::Mirror));
                let extend = button("Extend")
                    .width(Length::Fill)
                    .on_press(Message::SetMode(DisplayMode::Extend));
                let second_screen_only = button("Second screen only")
                    .width(Length::Fill)
                    .on_press(Message::SetMode(DisplayMode::SecondScreenOnly));
                let cancel = button("Cancel")
                    .width(Length::Fill)
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
}

impl Default for DisplaySwitcher {
    fn default() -> Self {
        Self::new()
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
            // Use the Exec variant to run a hyprctl command
            Dispatch::call(DispatchType::Exec(&format!(
                "hyprctl keyword monitor {} mirror,{}",
                secondary, primary
            )))
            .map_err(|e| e.to_string())?;
        }
        DisplayMode::Extend => {
            // Configure monitors for extended display
            let primary_mon = &monitors[0];
            let secondary_mon = &monitors[1];

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
