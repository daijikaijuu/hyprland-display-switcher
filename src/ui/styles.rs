use iced::widget::{button, container};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

pub fn main_container_style() -> impl Fn(&Theme) -> container::Style {
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

pub fn container_style() -> impl Fn(&Theme) -> container::Style {
    |_theme: &Theme| container::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        ..Default::default()
    }
}

pub fn card_button_style() -> impl Fn(&Theme, button::Status) -> button::Style {
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

pub fn cancel_button_style() -> impl Fn(&Theme, button::Status) -> button::Style {
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

pub fn reset_button_style() -> impl Fn(&Theme, button::Status) -> button::Style {
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

pub fn title_text_style() -> impl Fn(&Theme) -> iced::widget::text::Style {
    |_theme: &Theme| iced::widget::text::Style {
        color: Some(Color::from_rgb(0.95, 0.95, 0.95)),
    }
}

pub fn subtitle_text_style() -> impl Fn(&Theme) -> iced::widget::text::Style {
    |_theme: &Theme| iced::widget::text::Style {
        color: Some(Color::from_rgb(0.7, 0.7, 0.7)),
    }
}

pub fn card_title_text_style() -> impl Fn(&Theme) -> iced::widget::text::Style {
    |_theme: &Theme| iced::widget::text::Style {
        color: Some(Color::from_rgb(0.9, 0.9, 0.9)),
    }
}

pub fn card_description_text_style() -> impl Fn(&Theme) -> iced::widget::text::Style {
    |_theme: &Theme| iced::widget::text::Style {
        color: Some(Color::from_rgb(0.65, 0.65, 0.65)),
    }
}

pub fn settings_button_style() -> impl Fn(&Theme, button::Status) -> button::Style {
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

pub fn cancel_text_style() -> impl Fn(&Theme) -> iced::widget::text::Style {
    |_theme: &Theme| iced::widget::text::Style {
        color: Some(Color::from_rgb(0.9, 0.9, 0.9)),
    }
}
