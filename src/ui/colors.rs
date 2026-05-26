use crate::app::Theme;
use ratatui::style::Color;

pub struct Colors {
    pub bg: Color,
    pub text: Color,
    pub accent: Color,
    pub border: Color,
    pub ghost: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub error: Color,
    pub muted: Color,
}

impl Colors {
    pub fn from_theme(theme: &Theme) -> Self {
        match theme {
            Theme::Dark => Self {
                bg: Color::Rgb(18, 18, 18),
                text: Color::Rgb(220, 220, 220),
                accent: Color::Rgb(97, 175, 239),
                border: Color::Rgb(60, 60, 60),
                ghost: Color::Rgb(120, 120, 120),
                selected_bg: Color::Indexed(24),
                selected_fg: Color::Rgb(230, 230, 230),
                error: Color::Rgb(224, 108, 117),
                muted: Color::Rgb(100, 100, 100),
            },
            Theme::Light => Self {
                bg: Color::Rgb(250, 250, 250),
                text: Color::Rgb(30, 30, 30),
                accent: Color::Rgb(0, 100, 200),
                border: Color::Rgb(180, 180, 180),
                ghost: Color::Rgb(150, 150, 150),
                selected_bg: Color::Indexed(75),
                selected_fg: Color::Rgb(10, 10, 10),
                error: Color::Rgb(180, 40, 40),
                muted: Color::Rgb(130, 130, 130),
            },
        }
    }
}
