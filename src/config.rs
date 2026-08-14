use crossterm::style::Color;

pub struct ThemeColors {
    pub bg: Color,
    pub panel: Color,
    pub border: Color,
    pub title: Color,
    pub title_bg: Color,
    pub dir: Color,
    pub file: Color,
    pub sel_fg: Color,
    pub sel_bg: Color,
    pub muted: Color,
    pub accent: Color,
    pub search: Color,
    pub size: Color,
}

pub const THEME: ThemeColors = ThemeColors {
    bg: Color::Rgb { r: 24, g: 24, b: 29 },
    panel: Color::Rgb { r: 30, g: 30, b: 38 },
    border: Color::Rgb { r: 70, g: 72, b: 92 },
    title: Color::Rgb { r: 235, g: 235, b: 245 },
    title_bg: Color::Rgb { r: 48, g: 52, b: 80 },
    dir: Color::Rgb { r: 120, g: 170, b: 255 },
    file: Color::Rgb { r: 200, g: 200, b: 210 },
    sel_fg: Color::Rgb { r: 20, g: 22, b: 30 },
    sel_bg: Color::Rgb { r: 122, g: 200, b: 255 },
    muted: Color::Rgb { r: 120, g: 122, b: 140 },
    accent: Color::Rgb { r: 150, g: 220, b: 160 },
    search: Color::Rgb { r: 255, g: 214, b: 120 },
    size: Color::Rgb { r: 150, g: 152, b: 170 },
};

pub const PREVIEW_MAX_BYTES: u64 = 256 * 1024; // 256 KB
pub const PREVIEW_MAX_LINES: usize = 500;
