mod config;
mod display;

use config::{ConfigManager, ExtendLayout};
use display::{
    apply_extend_mode, apply_mirror_mode, apply_single_screen_mode, get_monitor_available_modes,
    reset_to_defaults,
};

use std::process;

use anyhow::Result;
use hyprland::data::{Monitor, Monitors};
use hyprland::shared::HyprData;
use iced::widget::{Space, button, column, container, pick_list, row, text};
use iced::{
    Background, Border, Color, Element, Event, Length, Padding, Shadow, Task, Theme, Vector,
    alignment, event, keyboard,
};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::{Application, to_layer_message};

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

struct DisplaySwitcher {
    state: State,
    config_manager: ConfigManager,
}

enum State {
    Loading,
    Loaded {
        monitors: Vec<Monitor>,
    },
    MonitorSettings {
        monitors: Vec<Monitor>,
        settings: MonitorSettingsData,
    },
    Error {
        message: String,
    },
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    LoadMonitors,
    MonitorsLoaded(Result<Vec<Monitor>, String>),
    SetMode(DisplayMode),
    OpenExtendSettings,
    UpdatePrimaryResolution(String),
    UpdatePrimaryRotation(String),
    UpdateSecondaryResolution(String),
    UpdateSecondaryRotation(String),
    UpdateLayout(ExtendLayout),
    UpdatePrimaryMonitor(String),
    ApplyExtendSettings,
    BackToMain,
    Cancel,
    ResetToDefaults,
    IcedEvent(Event),
}

#[derive(Debug, Clone)]
enum DisplayMode {
    Mirror,
    Extend,
    MainScreenOnly,
    SecondScreenOnly,
}

#[derive(Debug, Clone)]
struct MonitorSettingsData {
    primary_monitor: String,
    primary_resolution: String,
    primary_rotation: String,
    secondary_resolution: String,
    secondary_rotation: String,
    layout: ExtendLayout,
    primary_available_resolutions: Vec<String>,
    secondary_available_resolutions: Vec<String>,
    available_monitors: Vec<String>,
}

impl Default for MonitorSettingsData {
    fn default() -> Self {
        Self {
            primary_monitor: "".to_string(),
            primary_resolution: "auto".to_string(),
            primary_rotation: "normal".to_string(),
            secondary_resolution: "1920x1080".to_string(),
            secondary_rotation: "normal".to_string(),
            layout: ExtendLayout::LeftToRight,
            primary_available_resolutions: vec!["auto".to_string()],
            secondary_available_resolutions: vec!["1920x1080".to_string()],
            available_monitors: vec![],
        }
    }
}

impl Application for DisplaySwitcher {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Task<Message>) {
        let app = match Self::new() {
            Ok(app) => app,
            Err(e) => {
                eprintln!("Failed to initialize application: {e}");
                Self {
                    state: State::Error { message: e },
                    config_manager: ConfigManager::new().unwrap_or_default(),
                }
            }
        };

        (
            app,
            Task::perform(
                async {
                    Monitors::get()
                        .map(|monitors| monitors.into_iter().collect::<Vec<Monitor>>())
                        .map_err(|e| e.to_string())
                },
                Message::MonitorsLoaded,
            ),
        )
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
                    let result = match mode {
                        DisplayMode::Mirror => apply_mirror_mode(monitors),
                        DisplayMode::Extend => {
                            // Check if we have a saved configuration for these monitors
                            let monitor_names: Vec<String> =
                                monitors.iter().map(|m| m.name.clone()).collect();
                            if let Some(saved_config) = self
                                .config_manager
                                .get_extend_configuration_for_monitors(&monitor_names)
                            {
                                apply_extend_mode(monitors, saved_config)
                            } else {
                                // Create default extend configuration
                                if monitors.len() >= 2 {
                                    let default_config = ConfigManager::create_config_from_settings(
                                        monitors[0].name.clone(),
                                        monitors[1].name.clone(),
                                        format!("{}x{}", monitors[0].width, monitors[0].height),
                                        "normal".to_string(),
                                        format!("{}x{}", monitors[1].width, monitors[1].height),
                                        "normal".to_string(),
                                        ExtendLayout::LeftToRight,
                                    );
                                    apply_extend_mode(monitors, &default_config)
                                } else {
                                    Ok(())
                                }
                            }
                        }
                        DisplayMode::MainScreenOnly => apply_single_screen_mode(monitors, true),
                        DisplayMode::SecondScreenOnly => apply_single_screen_mode(monitors, false),
                    };

                    if let Err(e) = result {
                        eprintln!("Error applying display mode: {e}");
                    }
                }
                process::exit(0);
            }
            Message::OpenExtendSettings => {
                if let State::Loaded { monitors } = &self.state {
                    if monitors.len() < 2 {
                        return Task::none();
                    }

                    // Try to load saved configuration first
                    let monitor_names: Vec<String> =
                        monitors.iter().map(|m| m.name.clone()).collect();
                    let settings = if let Some(saved_config) = self
                        .config_manager
                        .get_extend_configuration_for_monitors(&monitor_names)
                    {
                        eprintln!(
                            "Loading saved configuration: Primary={}, Secondary={}",
                            saved_config.primary_monitor, saved_config.secondary_monitor
                        );

                        // Find which monitor is primary in saved config
                        let primary_monitor = &saved_config.primary_monitor;
                        let secondary_monitor = &saved_config.secondary_monitor;

                        // Get available resolutions for the actual primary and secondary monitors
                        let mut primary_available = vec!["auto".to_string()];
                        primary_available.extend(get_monitor_available_modes(primary_monitor));
                        let secondary_available = get_monitor_available_modes(secondary_monitor);

                        MonitorSettingsData {
                            primary_monitor: saved_config.primary_monitor.clone(),
                            primary_resolution: saved_config.primary_resolution.clone(),
                            primary_rotation: saved_config.primary_rotation.clone(),
                            secondary_resolution: saved_config.secondary_resolution.clone(),
                            secondary_rotation: saved_config.secondary_rotation.clone(),
                            layout: saved_config.layout.clone(),
                            primary_available_resolutions: primary_available,
                            secondary_available_resolutions: secondary_available,
                            available_monitors: monitors.iter().map(|m| m.name.clone()).collect(),
                        }
                    } else {
                        eprintln!(
                            "No saved configuration found, using defaults with {} as primary",
                            monitors[0].name
                        );
                        // Use default settings with first monitor as primary
                        let mut primary_available = vec!["auto".to_string()];
                        primary_available.extend(get_monitor_available_modes(&monitors[0].name));
                        let secondary_available = get_monitor_available_modes(&monitors[1].name);

                        MonitorSettingsData {
                            primary_monitor: monitors[0].name.clone(),
                            primary_resolution: "auto".to_string(),
                            primary_rotation: "normal".to_string(),
                            secondary_resolution: secondary_available
                                .first()
                                .cloned()
                                .unwrap_or_else(|| "1920x1080".to_string()),
                            secondary_rotation: "normal".to_string(),
                            layout: ExtendLayout::LeftToRight,
                            primary_available_resolutions: primary_available,
                            secondary_available_resolutions: secondary_available,
                            available_monitors: monitors.iter().map(|m| m.name.clone()).collect(),
                        }
                    };

                    self.state = State::MonitorSettings {
                        monitors: monitors.clone(),
                        settings,
                    };
                }
                Task::none()
            }
            Message::UpdatePrimaryResolution(resolution) => {
                if let State::MonitorSettings {
                    monitors: _,
                    settings,
                } = &mut self.state
                {
                    settings.primary_resolution = resolution;
                }
                Task::none()
            }
            Message::UpdatePrimaryRotation(rotation) => {
                if let State::MonitorSettings {
                    monitors: _,
                    settings,
                } = &mut self.state
                {
                    settings.primary_rotation = rotation;
                }
                Task::none()
            }
            Message::UpdateSecondaryResolution(resolution) => {
                if let State::MonitorSettings {
                    monitors: _,
                    settings,
                } = &mut self.state
                {
                    settings.secondary_resolution = resolution;
                }
                Task::none()
            }
            Message::UpdateSecondaryRotation(rotation) => {
                if let State::MonitorSettings {
                    monitors: _,
                    settings,
                } = &mut self.state
                {
                    settings.secondary_rotation = rotation;
                }
                Task::none()
            }
            Message::UpdateLayout(layout) => {
                if let State::MonitorSettings {
                    monitors: _,
                    settings,
                } = &mut self.state
                {
                    settings.layout = layout;
                }
                Task::none()
            }
            Message::UpdatePrimaryMonitor(monitor_name) => {
                if let State::MonitorSettings { monitors, settings } = &mut self.state {
                    settings.primary_monitor = monitor_name.clone();

                    // Update available resolutions for the new primary monitor
                    let primary_modes = get_monitor_available_modes(&monitor_name);
                    let mut primary_available = vec!["auto".to_string()];
                    primary_available.extend(primary_modes);
                    settings.primary_available_resolutions = primary_available;

                    // Update available resolutions for secondary monitor
                    if let Some(secondary_monitor) =
                        monitors.iter().find(|m| m.name != monitor_name)
                    {
                        settings.secondary_available_resolutions =
                            get_monitor_available_modes(&secondary_monitor.name);

                        // Reset resolution settings to defaults for the new configuration
                        settings.primary_resolution = "auto".to_string();
                        settings.secondary_resolution = settings
                            .secondary_available_resolutions
                            .first()
                            .cloned()
                            .unwrap_or_else(|| "1920x1080".to_string());
                    }
                }
                Task::none()
            }
            Message::ApplyExtendSettings => {
                if let State::MonitorSettings { monitors, settings } = &mut self.state {
                    // Find secondary monitor name
                    let secondary_monitor = monitors
                        .iter()
                        .find(|m| m.name != settings.primary_monitor)
                        .map(|m| m.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());

                    // Create and save configuration
                    let extend_config = ConfigManager::create_config_from_settings(
                        settings.primary_monitor.clone(),
                        secondary_monitor,
                        settings.primary_resolution.clone(),
                        settings.primary_rotation.clone(),
                        settings.secondary_resolution.clone(),
                        settings.secondary_rotation.clone(),
                        settings.layout.clone(),
                    );

                    // Save configuration
                    if let Err(e) = self
                        .config_manager
                        .save_extend_configuration(extend_config.clone())
                    {
                        eprintln!("Failed to save configuration: {e}");
                    }

                    // Apply the configuration
                    if let Err(e) = apply_extend_mode(monitors, &extend_config) {
                        eprintln!("Error applying extend mode settings: {e}");
                    }
                }
                process::exit(0);
            }
            Message::BackToMain => {
                if let State::MonitorSettings { monitors, .. } = &self.state {
                    self.state = State::Loaded {
                        monitors: monitors.clone(),
                    };
                }
                Task::none()
            }
            Message::Cancel => {
                process::exit(0);
            }
            Message::ResetToDefaults => {
                if let Err(e) = reset_to_defaults() {
                    eprintln!("Error resetting to defaults: {e}");
                }
                process::exit(0);
            }
            Message::IcedEvent(Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            })) => {
                process::exit(0);
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
            State::Error { message } => container(text(format!("Error: {message}")).size(16))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .style(container_style())
                .into(),

            State::MonitorSettings { monitors, settings } => {
                self.create_monitor_settings_view(monitors, settings)
            }

            State::Loaded { monitors } => {
                let title = text("Choose display mode")
                    .size(28)
                    .style(title_text_style());

                let subtitle = text(format!(
                    "{} display{} detected",
                    monitors.len(),
                    if monitors.len() == 1 { "" } else { "s" }
                ))
                .size(14)
                .style(subtitle_text_style());

                let pc_screen_card = create_display_card(
                    "💻".to_string(),
                    "PC screen only".to_string(),
                    "Use only your main display".to_string(),
                    Message::SetMode(DisplayMode::MainScreenOnly),
                );

                let duplicate_card = create_display_card(
                    "📱".to_string(),
                    "Duplicate displays".to_string(),
                    "Show the same content on all displays".to_string(),
                    Message::SetMode(DisplayMode::Mirror),
                );

                let extend_card = create_extend_card();

                let second_screen_card = create_display_card(
                    "📺".to_string(),
                    "Second screen only".to_string(),
                    "Use only your external display".to_string(),
                    Message::SetMode(DisplayMode::SecondScreenOnly),
                );

                let cancel_button = button(
                    container(text("Cancel").size(16).style(cancel_text_style()))
                        .padding(Padding::from([12, 24]))
                        .align_x(alignment::Horizontal::Center),
                )
                .width(Length::Fill)
                .style(cancel_button_style())
                .on_press(Message::Cancel);

                let reset_button = button(
                    container(text("Reset").size(16).style(cancel_text_style()))
                        .padding(Padding::from([12, 24]))
                        .align_x(alignment::Horizontal::Center),
                )
                .width(Length::Fill)
                .style(reset_button_style())
                .on_press(Message::ResetToDefaults);

                container(
                    column![
                        title,
                        subtitle,
                        Space::with_height(16),
                        pc_screen_card,
                        duplicate_card,
                        extend_card,
                        second_screen_card,
                        Space::with_height(16),
                        row![cancel_button, reset_button].spacing(12)
                    ]
                    .spacing(12)
                    .padding(24)
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Center),
                )
                .width(480)
                .style(main_container_style())
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .padding(Padding::from(16))
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
    fn new() -> Result<Self, String> {
        let config_manager = ConfigManager::new()
            .map_err(|e| format!("Failed to initialize config manager: {e}"))?;

        Ok(Self {
            state: State::Loading,
            config_manager,
        })
    }

    fn create_monitor_settings_view<'a>(
        &self,
        monitors: &[Monitor],
        settings: &'a MonitorSettingsData,
    ) -> Element<'a, Message> {
        let title = text("Monitor Settings - Extend Mode")
            .size(24)
            .style(title_text_style());

        // Determine actual primary and secondary monitor names based on user selection
        let primary_monitor_name = &settings.primary_monitor;
        let secondary_monitor_name = monitors
            .iter()
            .find(|m| m.name != settings.primary_monitor)
            .map(|m| m.name.as_str())
            .unwrap_or("Secondary");

        let primary_section = column![
            text(format!("Primary Monitor ({primary_monitor_name})"))
                .size(16)
                .style(card_title_text_style()),
            row![
                column![
                    text("Resolution:").size(12),
                    pick_list(
                        settings.primary_available_resolutions.as_slice(),
                        settings
                            .primary_available_resolutions
                            .iter()
                            .find(|&r| r == &settings.primary_resolution),
                        |res| Message::UpdatePrimaryResolution(res.clone())
                    )
                    .width(120)
                ]
                .spacing(4),
                column![
                    text("Rotation:").size(12),
                    pick_list(
                        vec!["normal", "left", "right", "inverted"],
                        Some(settings.primary_rotation.as_str()),
                        |rot| Message::UpdatePrimaryRotation(rot.to_string())
                    )
                    .width(100)
                ]
                .spacing(4)
            ]
            .spacing(16)
        ]
        .spacing(8);

        let secondary_section = column![
            text(format!("Secondary Monitor ({secondary_monitor_name})"))
                .size(16)
                .style(card_title_text_style()),
            row![
                column![
                    text("Resolution:").size(12),
                    pick_list(
                        settings.secondary_available_resolutions.as_slice(),
                        settings
                            .secondary_available_resolutions
                            .iter()
                            .find(|&r| r == &settings.secondary_resolution),
                        |res| Message::UpdateSecondaryResolution(res.clone())
                    )
                    .width(120)
                ]
                .spacing(4),
                column![
                    text("Rotation:").size(12),
                    pick_list(
                        vec!["normal", "left", "right", "inverted"],
                        Some(settings.secondary_rotation.as_str()),
                        |rot| Message::UpdateSecondaryRotation(rot.to_string())
                    )
                    .width(100)
                ]
                .spacing(4)
            ]
            .spacing(16)
        ]
        .spacing(8);

        let primary_monitor_section = column![
            text("Primary Monitor:")
                .size(16)
                .style(card_title_text_style()),
            pick_list(
                settings.available_monitors.as_slice(),
                settings
                    .available_monitors
                    .iter()
                    .find(|&m| m == &settings.primary_monitor),
                |monitor| Message::UpdatePrimaryMonitor(monitor.clone())
            )
            .width(200)
        ]
        .spacing(8);

        let layout_section = column![
            text("Layout:").size(16).style(card_title_text_style()),
            pick_list(
                vec![
                    ExtendLayout::LeftToRight,
                    ExtendLayout::RightToLeft,
                    ExtendLayout::TopToBottom,
                    ExtendLayout::BottomToTop
                ],
                Some(&settings.layout),
                Message::UpdateLayout
            )
            .width(200)
        ]
        .spacing(8);

        let buttons = row![
            button(
                container(text("Back").size(14))
                    .padding(Padding::from([8, 16]))
                    .align_x(alignment::Horizontal::Center)
            )
            .style(cancel_button_style())
            .on_press(Message::BackToMain),
            button(
                container(text("Reset").size(14))
                    .padding(Padding::from([8, 16]))
                    .align_x(alignment::Horizontal::Center)
            )
            .style(reset_button_style())
            .on_press(Message::ResetToDefaults),
            button(
                container(text("Apply Settings").size(14))
                    .padding(Padding::from([8, 16]))
                    .align_x(alignment::Horizontal::Center)
            )
            .style(card_button_style())
            .on_press(Message::ApplyExtendSettings)
        ]
        .spacing(12);

        container(
            column![
                title,
                Space::with_height(16),
                primary_monitor_section,
                Space::with_height(16),
                primary_section,
                Space::with_height(16),
                secondary_section,
                Space::with_height(16),
                layout_section,
                Space::with_height(20),
                buttons
            ]
            .spacing(8)
            .padding(24)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
        )
        .width(500)
        .style(main_container_style())
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .padding(Padding::from(16))
        .into()
    }
}

fn create_extend_card() -> Element<'static, Message> {
    let card_content = container(
        row![
            container(text("🖥️").size(32))
                .width(60)
                .align_x(alignment::Horizontal::Center),
            column![
                text("Extend displays")
                    .size(18)
                    .style(card_title_text_style()),
                text("Use displays as one continuous workspace")
                    .size(13)
                    .style(card_description_text_style())
            ]
            .spacing(4)
            .width(Length::Fill),
            button(
                container(text("⚙️").size(16))
                    .padding(Padding::from([4, 8]))
                    .align_x(alignment::Horizontal::Center)
            )
            .style(settings_button_style())
            .on_press(Message::OpenExtendSettings)
        ]
        .spacing(12)
        .align_y(alignment::Vertical::Center),
    )
    .padding(Padding::from([16, 20]))
    .width(Length::Fill);

    button(card_content)
        .width(Length::Fill)
        .style(card_button_style())
        .on_press(Message::SetMode(DisplayMode::Extend))
        .into()
}

fn create_display_card(
    icon: String,
    title: String,
    description: String,
    message: Message,
) -> Element<'static, Message> {
    let card_content = container(
        row![
            container(text(icon).size(32))
                .width(60)
                .align_x(alignment::Horizontal::Center),
            column![
                text(title).size(18).style(card_title_text_style()),
                text(description)
                    .size(13)
                    .style(card_description_text_style())
            ]
            .spacing(4)
            .width(Length::Fill)
        ]
        .spacing(16)
        .align_y(alignment::Vertical::Center),
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

fn reset_button_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_theme: &Theme, status: button::Status| {
        let background_color = match status {
            button::Status::Hovered => Color::from_rgba(0.6, 0.4, 0.2, 0.8),
            button::Status::Pressed => Color::from_rgba(0.55, 0.35, 0.15, 0.9),
            _ => Color::from_rgba(0.3, 0.3, 0.2, 0.8),
        };

        let border_color = match status {
            button::Status::Hovered => Color::from_rgba(0.7, 0.5, 0.3, 0.8),
            button::Status::Pressed => Color::from_rgba(0.65, 0.45, 0.25, 0.9),
            _ => Color::from_rgba(0.4, 0.4, 0.3, 0.6),
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

fn settings_button_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_theme: &Theme, status: button::Status| {
        let background_color = match status {
            button::Status::Hovered => Color::from_rgba(0.4, 0.4, 0.4, 0.8),
            button::Status::Pressed => Color::from_rgba(0.35, 0.35, 0.35, 0.9),
            _ => Color::from_rgba(0.25, 0.25, 0.25, 0.7),
        };

        let border_color = match status {
            button::Status::Hovered => Color::from_rgba(0.5, 0.5, 0.5, 0.8),
            button::Status::Pressed => Color::from_rgba(0.45, 0.45, 0.45, 0.9),
            _ => Color::from_rgba(0.35, 0.35, 0.35, 0.6),
        };

        button::Style {
            background: Some(Background::Color(background_color)),
            border: Border {
                radius: 6.into(),
                width: 1.0,
                color: border_color,
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        }
    }
}

fn cancel_text_style() -> impl Fn(&Theme) -> iced::widget::text::Style {
    |_theme: &Theme| iced::widget::text::Style {
        color: Some(Color::from_rgb(0.9, 0.9, 0.9)),
    }
}
