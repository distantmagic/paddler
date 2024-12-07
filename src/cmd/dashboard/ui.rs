use ratatui::style::{palette::tailwind, Color};

#[derive(Clone, Copy)]
pub struct TableColors {
    pub buffer_bg: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub normal_row_color: Color,
    pub row_fg: Color,
    pub selected_row_style_fg: Color,
}

impl TableColors {
    pub const fn new() -> Self {
        Self {
            buffer_bg: tailwind::NEUTRAL.c950,
            header_bg: tailwind::NEUTRAL.c950,
            header_fg: tailwind::NEUTRAL.c950,
            row_fg: tailwind::NEUTRAL.c950,
            selected_row_style_fg: tailwind::NEUTRAL.c400,
            normal_row_color: tailwind::NEUTRAL.c950,
        }
    }
}
