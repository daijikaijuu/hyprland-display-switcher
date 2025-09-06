use crate::state::{DisplayMode, Message};
use iced::widget::{button, column, container, row, text};
use iced::{alignment, Element, Font, Length, Padding};

// Font constants for emoji display
const EMOJI_FONT_NOTO: Font = Font::with_name("Noto Color Emoji");

// Create a special function for monitor emoji
fn create_monitor_emoji() -> iced::widget::Text<'static> {
    text("🖥️").size(32).font(EMOJI_FONT_NOTO)
}

// Special function for selection arrow
fn create_selection_arrow() -> iced::widget::Text<'static> {
    text("▶").size(16).font(EMOJI_FONT_NOTO)
}

// Create emoji text for dynamic content
fn create_emoji_text_dynamic(emoji: String, size: u16) -> iced::widget::Text<'static> {
    text(emoji).size(size).font(EMOJI_FONT_NOTO)
}

pub fn create_extend_card_with_selection(is_selected: bool) -> Element<'static, Message> {
    let title_content: Element<'static, Message> = if is_selected {
        row![
            create_selection_arrow(),
            text(" Extend displays (3)")
                .size(18)
                .style(crate::ui::card_title_text_style()),
        ]
        .align_y(alignment::Vertical::Center)
        .into()
    } else {
        text("Extend displays (3)")
            .size(18)
            .style(crate::ui::card_title_text_style())
            .into()
    };

    // Use the special monitor emoji function
    let monitor_emoji = create_monitor_emoji();

    let card_content = container(
        row![
            container(monitor_emoji)
                .width(60)
                .align_x(alignment::Horizontal::Center),
            column![
                title_content,
                text("Use displays as one continuous workspace")
                    .size(13)
                    .style(crate::ui::card_description_text_style())
            ]
            .spacing(4)
            .width(Length::Fill),
            button(
                container(text("⚙️").size(16).font(EMOJI_FONT_NOTO))
                    .padding(Padding::from([4, 8]))
                    .align_x(alignment::Horizontal::Center)
            )
            .style(crate::ui::settings_button_style())
            .on_press(Message::OpenExtendSettings)
        ]
        .spacing(12)
        .align_y(alignment::Vertical::Center),
    )
    .padding(Padding::from([16, 20]))
    .width(Length::Fill);

    button(card_content)
        .width(Length::Fill)
        .style(crate::ui::card_button_style_with_selection(is_selected))
        .on_press(Message::SetMode(DisplayMode::Extend))
        .into()
}

pub fn create_display_card_with_selection(
    icon: String,
    title: String,
    description: String,
    message: Message,
    is_selected: bool,
) -> Element<'static, Message> {
    let title_content: Element<'static, Message> = if is_selected {
        row![
            create_selection_arrow(),
            text(format!(" {}", title))
                .size(18)
                .style(crate::ui::card_title_text_style()),
        ]
        .align_y(alignment::Vertical::Center)
        .into()
    } else {
        text(title)
            .size(18)
            .style(crate::ui::card_title_text_style())
            .into()
    };

    let card_content = container(
        row![
            container(create_emoji_text_dynamic(icon, 32))
                .width(60)
                .align_x(alignment::Horizontal::Center),
            column![
                title_content,
                text(description)
                    .size(13)
                    .style(crate::ui::card_description_text_style())
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
        .style(crate::ui::card_button_style_with_selection(is_selected))
        .on_press(message)
        .into()
}
