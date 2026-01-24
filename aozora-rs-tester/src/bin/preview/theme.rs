use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub const HEADER_BG: Color = Color::Reset;
    pub const HEADER_FG_TITLE: Color = Color::Cyan;
    pub const HEADER_FG_FILE: Color = Color::Yellow;
    pub const HEADER_FG_INFO: Color = Color::DarkGray;

    pub const STATUS_BAR_FG: Color = Color::Gray;

    pub const LIST_SELECTED_FG: Color = Color::Yellow;
    pub const LIST_SELECTED_MODIFIER: Modifier = Modifier::BOLD;

    pub const ERROR_FG: Color = Color::Red;
    pub const SUCCESS_FG: Color = Color::Green;

    pub const TEXT_LINENUM_FG: Color = Color::DarkGray;

    pub fn selected_style() -> Style {
        Style::default()
            .fg(Self::LIST_SELECTED_FG)
            .add_modifier(Self::LIST_SELECTED_MODIFIER)
    }

    pub fn highlight_block_style() -> Style {
        Style::default().add_modifier(Modifier::REVERSED)
    }
}
