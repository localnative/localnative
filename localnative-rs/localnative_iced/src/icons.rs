use iced::Font;
use iced::{widget::Text, Element};

pub const ICONS: Font = Font::with_name("remixicon");

pub enum IconItem {
    Search,
    Clear,
    Delete,
    Settings,
    Filter,
    FilterOff,
    Refresh,
    Next,
    Pre,
    Full,
    FullExit,
    QRCode,
    DayTime,
    MonthTime,
    SyncFromFile,
    SyncFromServer,
    SyncToServer,
    OpenServer,
    CloseServer,
    Sync,
    Dark,
    Light,
    Date,
    Note,
}

impl IconItem {
    fn to_char(&self) -> char {
        match self {
            IconItem::Search => '\u{f0d1}',
            IconItem::Clear => '\u{eb99}',
            IconItem::Delete => '\u{ec1e}',
            IconItem::Settings => '\u{f0e6}',
            IconItem::Filter => '\u{ed27}',
            IconItem::FilterOff => '\u{ed29}',
            IconItem::Refresh => '\u{ed2a}',
            IconItem::Next => '\u{ea6e}',
            IconItem::Pre => '\u{ea64}',
            IconItem::Full => '\u{ed9c}',
            IconItem::FullExit => '\u{ed9a}',
            IconItem::QRCode => '\u{f03d}',
            IconItem::DayTime => '\u{f20f}',
            IconItem::MonthTime => '\u{f20e}',
            IconItem::SyncFromFile => '\u{eccf}',
            IconItem::SyncFromServer => '\u{ec58}',
            IconItem::SyncToServer => '\u{f24d}',
            IconItem::OpenServer => '\u{eb9d}',
            IconItem::CloseServer => '\u{eb9f}',
            IconItem::Sync => '\u{eba1}',
            IconItem::Dark => '\u{ef72}',
            IconItem::Light => '\u{f1bf}',
            IconItem::Date => '\u{eb29}',
            IconItem::Note => '\u{ea7e}',
        }
    }

    pub fn to_text<'text>(self) -> Text<'text> {
        Text::new(self.to_char().to_string()).size(25).font(ICONS)
    }
}

impl<'a, Message> From<IconItem> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(icon: IconItem) -> Element<'a, Message> {
        Element::new(icon.to_text())
    }
}
