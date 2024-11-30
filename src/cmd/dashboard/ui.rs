use ratatui::style::{palette::tailwind, Color};

#[derive(Clone, Copy)]
pub struct TableColors {
    pub buffer_bg: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub row_fg: Color,
    pub selected_row_style_fg: Color,
    pub normal_row_color: Color,
    pub footer_border_color: Color,
}

impl TableColors {
    pub const fn new() -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: tailwind::SLATE.c950,
            header_fg: tailwind::SLATE.c950,
            row_fg: tailwind::SLATE.c950,
            selected_row_style_fg: tailwind::GRAY.c400,
            normal_row_color: tailwind::SLATE.c950,
            footer_border_color: tailwind::SLATE.c950,
        }
    }
}
