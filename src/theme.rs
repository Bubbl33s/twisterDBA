use ratatui::style::Color;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Theme {
    pub bg: Color,
    pub editor_bg: Color,
    pub keyword: Color,
    pub string: Color,
    pub number: Color,
    pub comment: Color,
    pub identifier: Color,
    pub operator: Color,
    pub statusline_active_bg: Color,
    pub statusline_inactive_bg: Color,
    pub status_connected: Color,
    pub status_connecting: Color,
    pub status_error: Color,
    pub status_disconnected: Color,
    pub dialog_type_selected_bg: Color,
    pub dialog_profile_bg: Color,
    pub dialog_backdrop_dim: Color,
    pub dialog_cursor_bg: Color,
    pub dialog_cursor_fg: Color,
    pub dialog_field_active_bg: Color,
    pub icons: IconMap,
    pub nerd_font_available: bool,
}

impl Theme {
    pub fn darcula() -> Self {
        let nerd_font_available = std::env::var("NERD_FONT")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or_else(|_| {
                std::env::var("TERM")
                    .map(|t| {
                        t.contains("kitty") || t.contains("alacritty") || t.contains("wezterm")
                    })
                    .unwrap_or(false)
            });
        Self {
            bg: Color::Rgb(43, 43, 43),
            editor_bg: Color::Rgb(30, 31, 34),
            keyword: Color::Rgb(204, 120, 50),
            string: Color::Rgb(106, 135, 89),
            number: Color::Rgb(104, 151, 187),
            comment: Color::Rgb(128, 128, 128),
            identifier: Color::Rgb(169, 183, 198),
            operator: Color::Rgb(204, 120, 50),
            statusline_active_bg: Color::Rgb(58, 110, 165),
            statusline_inactive_bg: Color::Rgb(60, 63, 65),
            status_connected: Color::Rgb(80, 200, 80),
            status_connecting: Color::Rgb(230, 200, 50),
            status_error: Color::Rgb(220, 60, 60),
            status_disconnected: Color::Rgb(128, 128, 128),
            dialog_type_selected_bg: Color::Rgb(75, 110, 165),
            dialog_profile_bg: Color::Rgb(60, 63, 65),
            dialog_backdrop_dim: Color::Rgb(10, 10, 10),
            dialog_cursor_bg: Color::Rgb(169, 183, 198),
            dialog_cursor_fg: Color::Rgb(30, 31, 34),
            dialog_field_active_bg: Color::Rgb(60, 63, 65),
            icons: IconMap::darcula(),
            nerd_font_available,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct IconMap {
    pub database: (char, Color),
    pub schema: (char, Color),
    pub table: (char, Color),
    pub view: (char, Color),
    pub routine: (char, Color),
    pub column: (char, Color),
    pub postgres: (char, Color),
    pub mysql: (char, Color),
    pub sqlite: (char, Color),
    pub folder: (char, Color),
    pub key: (char, Color),
    pub foreign_key: (char, Color),
    pub index: (char, Color),
}

impl IconMap {
    pub fn darcula() -> Self {
        Self {
            database: ('\u{F06FC}', Color::Rgb(77, 182, 172)),
            schema: ('\u{F07C0}', Color::Rgb(255, 203, 107)),
            table: ('\u{F021A}', Color::Rgb(84, 138, 247)),
            view: ('\u{F0219}', Color::Rgb(186, 104, 200)),
            routine: ('\u{F0B21}', Color::Rgb(229, 115, 115)),
            column: ('\u{F071A}', Color::Rgb(169, 183, 198)),
            postgres: ('\u{F06FC}', Color::Rgb(77, 182, 172)),
            mysql: ('\u{F07C0}', Color::Rgb(84, 138, 247)),
            sqlite: ('\u{F021A}', Color::Rgb(169, 183, 198)),
            folder: ('\u{F0248}', Color::Rgb(255, 203, 107)),
            key: ('\u{F093D}', Color::Rgb(255, 203, 107)),
            foreign_key: ('\u{F0337}', Color::Rgb(229, 155, 90)),
            index: ('\u{F018A}', Color::Rgb(84, 138, 247)),
        }
    }
}
