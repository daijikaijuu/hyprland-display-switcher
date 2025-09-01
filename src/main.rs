use std::process;

use anyhow::Result;
use hyprland::data::{Monitor, Monitors};
use hyprland::dispatch::{Dispatch, DispatchType};
use hyprland::shared::HyprData;
use iced::widget::{button, column, container, text, row, Space};
use iced::{Element, Length, Task, Theme, alignment, event, Event, Padding, Color, Border, Background, Shadow, Vector};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::{Application, to_layer_message};

fn main() -> Result<(), iced_layershell::Error> {
    DisplaySwitcher::run(Settings {
        layer_settings: LayerShellSettings {
            size: Some((500, 700)),
            exclusive_zone: 0,
            anchor: Anchor::empty(),
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
            State::Loading => container(text("Loading display information...").size(20))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .style(container_style())
                .into(),
            State::Error { message } => container(text(format!("Error: {}", message)).size(16))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .style(container_style())
                .into(),

            State::Loaded { monitors } => {
                let title = text("Choose display mode")
                    .size(28)
                    .style(title_text_style());
                
                let subtitle = text(format!("{} display{} detected", monitors.len(), if monitors.len() == 1 { "" } else { "s" }))
                    .size(14)
                    .style(subtitle_text_style());

                let pc_screen_card = create_display_card(
                    "💻".to_string(), 
                    "PC screen only".to_string(), 
                    "Use only your main display".to_string(),
                    Message::SetMode(DisplayMode::MainScreenOnly)
                );
                
                let duplicate_card = create_display_card(
                    "📱".to_string(), 
                    "Duplicate displays".to_string(), 
                    "Show the same content on all displays".to_string(),
                    Message::SetMode(DisplayMode::Mirror)
                );
                
                let extend_card = create_display_card(
                    "🖥️".to_string(), 
                    "Extend displays".to_string(), 
                    "Use displays as one continuous workspace".to_string(),
                    Message::SetMode(DisplayMode::Extend)
                );
                
                let second_screen_card = create_display_card(
                    "📺".to_string(), 
                    "Second screen only".to_string(), 
                    "Use only your external display".to_string(),
                    Message::SetMode(DisplayMode::SecondScreenOnly)
                );

                let cancel_button = button(
                    container(
                        text("Cancel")
                            .size(16)
                            .style(cancel_text_style())
                    )
                    .padding(Padding::from([12, 24]))
                    .align_x(alignment::Horizontal::Center)
                )
                .width(Length::Fill)
                .style(cancel_button_style())
                .on_press(Message::Cancel);

                container(
                    column![
                        title,
                        subtitle,
                        Space::with_height(20),
                        pc_screen_card,
                        duplicate_card,
                        extend_card,
                        second_screen_card,
                        Space::with_height(20),
                        cancel_button
                    ]
                    .spacing(16)
                    .padding(32)
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Center)
                )
                .width(480)
                .style(main_container_style())
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Top)
                .padding(Padding::from(20))
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

fn create_display_card(icon: String, title: String, description: String, message: Message) -> Element<'static, Message> {
    let card_content = container(
        row![
            container(text(icon).size(32))
                .width(60)
                .align_x(alignment::Horizontal::Center),
            column![
                text(title)
                    .size(18)
                    .style(card_title_text_style()),
                text(description)
                    .size(13)
                    .style(card_description_text_style())
            ]
            .spacing(4)
            .width(Length::Fill)
        ]
        .spacing(16)
        .align_y(alignment::Vertical::Center)
    )
    .padding(Padding::from([16, 20]))
    .width(Length::Fill);

    button(card_content)
        .width(Length::Fill)
        .style(card_button_style())
        .on_press(message)
        .into()
}

fn main_container_style() -> impl Fn(&Theme) -> container::Style {
    |_theme: &Theme| container::Style {
        background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.95))),
        border: Border {
            radius: 16.into(),
            width: 1.0,
            color: Color::from_rgba(0.3, 0.3, 0.3, 0.5),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    }
}

fn container_style() -> impl Fn(&Theme) -> container::Style {
    |_theme: &Theme| container::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        ..Default::default()
    }
}

fn card_button_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_theme: &Theme, status: button::Status| {
        let background_color = match status {
            button::Status::Hovered => Color::from_rgba(0.2, 0.4, 0.7, 0.8),
            button::Status::Pressed => Color::from_rgba(0.15, 0.35, 0.65, 0.9),
            _ => Color::from_rgba(0.15, 0.15, 0.15, 0.9),
        };

        let border_color = match status {
            button::Status::Hovered => Color::from_rgba(0.3, 0.5, 0.8, 0.8),
            button::Status::Pressed => Color::from_rgba(0.25, 0.45, 0.75, 0.9),
            _ => Color::from_rgba(0.3, 0.3, 0.3, 0.6),
        };

        button::Style {
            background: Some(Background::Color(background_color)),
            border: Border {
                radius: 12.into(),
                width: 1.0,
                color: border_color,
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        }
    }
}

fn cancel_button_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_theme: &Theme, status: button::Status| {
        let background_color = match status {
            button::Status::Hovered => Color::from_rgba(0.7, 0.2, 0.2, 0.8),
            button::Status::Pressed => Color::from_rgba(0.65, 0.15, 0.15, 0.9),
            _ => Color::from_rgba(0.2, 0.2, 0.2, 0.8),
        };

        let border_color = match status {
            button::Status::Hovered => Color::from_rgba(0.8, 0.3, 0.3, 0.8),
            button::Status::Pressed => Color::from_rgba(0.75, 0.25, 0.25, 0.9),
            _ => Color::from_rgba(0.4, 0.4, 0.4, 0.6),
        };

        button::Style {
            background: Some(Background::Color(background_color)),
            border: Border {
                radius: 8.into(),
                width: 1.0,
                color: border_color,
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 4.0,
            },
            ..Default::default()
        }
    }
}

fn title_text_style() -> impl Fn(&Theme) -> iced::widget::text::Style {
    |_theme: &Theme| iced::widget::text::Style {
        color: Some(Color::from_rgb(0.95, 0.95, 0.95)),
    }
}

fn subtitle_text_style() -> impl Fn(&Theme) -> iced::widget::text::Style {
    |_theme: &Theme| iced::widget::text::Style {
        color: Some(Color::from_rgb(0.7, 0.7, 0.7)),
    }
}

fn card_title_text_style() -> impl Fn(&Theme) -> iced::widget::text::Style {
    |_theme: &Theme| iced::widget::text::Style {
        color: Some(Color::from_rgb(0.9, 0.9, 0.9)),
    }
}

fn card_description_text_style() -> impl Fn(&Theme) -> iced::widget::text::Style {
    |_theme: &Theme| iced::widget::text::Style {
        color: Some(Color::from_rgb(0.65, 0.65, 0.65)),
    }
}

fn cancel_text_style() -> impl Fn(&Theme) -> iced::widget::text::Style {
    |_theme: &Theme| iced::widget::text::Style {
        color: Some(Color::from_rgb(0.9, 0.9, 0.9)),
    }
}