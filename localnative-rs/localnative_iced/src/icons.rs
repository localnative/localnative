use iced::{Font, Text};

pub const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/icons.ttf"),
};

pub struct Icons;

impl Icons {
    // .ri-search-line:before { content: "\f0d1"; }
    // .ri-close-circle-fill:before { content: "\eb96"; }
    // .ri-delete-back-2-fill:before { content: "\ec19"; }
    // .ri-delete-bin-2-fill:before { content: "\ec1d"; }
    // .ri-settings-4-line:before { content: "\f0e8"; }
    // .ri-filter-line:before { content: "\ed27"; }
    // .ri-refresh-fill:before { content: "\f063"; }
    // .ri-arrow-right-s-line:before { content: "\ea6e"; }
    // .ri-arrow-left-s-line:before { content: "\ea64"; }
    // .ri-fullscreen-line:before { content: "\ed9c"; }
    // .ri-fullscreen-exit-fill:before { content: "\ed99"; }
    pub fn search() -> char {
        '\u{f0d1}'
    }
    pub fn close() -> char {
        '\u{eb96}'
    }
    pub fn delete_back() -> char {
        '\u{ec19}'
    }
    pub fn delete_bin() -> char {
        '\u{ec1d}'
    }
    pub fn settings() -> char {
        '\u{f0e8}'
    }
    pub fn filter() -> char {
        '\u{ed27}'
    }
    pub fn refresh() -> char {
        '\u{f063}'
    }
    pub fn next() -> char {
        '\u{ea6e}'
    }
    pub fn pre() -> char {
        '\u{ea64}'
    }
    pub fn full() -> char {
        '\u{ed9c}'
    }
    pub fn full_exit() -> char {
        '\u{ed99}'
    }
    pub fn qr_code() -> char {
        '\u{f03d}'
    }
    pub fn time() -> char {
        '\u{f20e}'
    }
    pub fn filter_off() -> char {
        '\u{ed29}'
    }
}

pub fn icon(unicode: char) -> Text {
    Text::new(&unicode.to_string()).font(ICONS)
}
