use crate::state::{DisplayMode, Message};
use iced::widget::{button, column, container, row, text};
use iced::{alignment, Element, Font, Length, Padding};

const EMOJI_FONT: Font = Font::with_name("Noto Color Emoji");

pub fn create_extend_card() -> Element<'static, Message> {
    let card_content = container(
        row![
            container(text("🖥️").size(32).font(EMOJI_FONT))
                .width(60)
                .align_x(alignment::Horizontal::Center),
            column![
                text("Extend displays")
                    .size(18)
                    .style(crate::ui::card_title_text_style()),
                text("Use displays as one continuous workspace")
                    .size(13)
                    .style(crate::ui::card_description_text_style())
            ]
            .spacing(4)
            .width(Length::Fill),
            button(
                container(text("⚙️").size(16).font(EMOJI_FONT))
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
        .style(crate::ui::card_button_style())
        .on_press(Message::SetMode(DisplayMode::Extend))
        .into()
}

pub fn create_display_card(
    icon: String,
    title: String,
    description: String,
    message: Message,
) -> Element<'static, Message> {
    let card_content = container(
        row![
            container(text(icon).size(32).font(EMOJI_FONT))
                .width(60)
                .align_x(alignment::Horizontal::Center),
            column![
                text(title)
                    .size(18)
                    .style(crate::ui::card_title_text_style()),
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
        .style(crate::ui::card_button_style())
        .on_press(message)
        .into()
}
