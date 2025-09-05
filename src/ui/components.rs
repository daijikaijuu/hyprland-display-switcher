use crate::state::{DisplayMode, Message};
use iced::widget::{button, column, container, row, text};
use iced::{alignment, Element, Font, Length, Padding};

// Font fallback system for emoji display
const EMOJI_FONTS: [Font; 4] = [
    Font::with_name("Noto Color Emoji"),
    Font::with_name("FiraCode Nerd Font"),
    Font::with_name("FiraCode Nerd Font Mono"),
    Font::with_name("Segoe UI Emoji"),
];

// Emoji mappings for better font compatibility
fn get_best_font_for_emoji(emoji: &str) -> Font {
    match emoji {
        "⚙️" | "🔧" | "🛠️" => EMOJI_FONTS[1], // Use Nerd Font for technical symbols
        "🖥️" | "💻" | "📱" | "📺" => EMOJI_FONTS[0], // Use Noto Color Emoji for devices
        _ => EMOJI_FONTS[0],                  // Default to Noto Color Emoji
    }
}

fn create_emoji_text(emoji: &'static str, size: u16) -> iced::widget::Text<'static> {
    let best_font = get_best_font_for_emoji(emoji);
    text(emoji).size(size).font(best_font)
}

fn create_emoji_text_dynamic(emoji: String, size: u16) -> iced::widget::Text<'static> {
    let best_font = get_best_font_for_emoji(&emoji);
    text(emoji).size(size).font(best_font)
}

pub fn create_extend_card() -> Element<'static, Message> {
    let card_content = container(
        row![
            container(create_emoji_text("🖥️", 32))
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
                container(create_emoji_text("⚙️", 16))
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
            container(create_emoji_text_dynamic(icon, 32))
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
