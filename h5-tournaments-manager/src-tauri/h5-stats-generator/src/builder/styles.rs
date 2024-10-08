use lazy_static::lazy_static;
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder};

lazy_static! {
    pub static ref THIN_BORDER: Format = Format::new().set_border(FormatBorder::Thin);
    pub static ref THIN_BORDER_TEXT_CENTER: Format = Format::new().set_border(FormatBorder::Thin).set_align(FormatAlign::Center);
    pub static ref THIN_BORDER_TEXP_WRAP: Format = Format::new().set_border(FormatBorder::Thin).set_align(FormatAlign::Center).set_text_wrap();
    pub static ref TEXT_CENTER_COLOR_RED: Format = Format::new().set_align(FormatAlign::VerticalCenter).set_align(FormatAlign::Center).set_background_color(Color::Red);
    pub static ref TEXT_BOLD_CENTERED: Format = Format::new().set_align(FormatAlign::Center).set_align(FormatAlign::CenterAcross).set_bold().set_text_wrap().set_border(FormatBorder::Thin);
    pub static ref BACKGROUND_SILVER: Format = Format::new().set_border(FormatBorder::Thin).set_background_color(Color::Silver);
    pub static ref BACKGROUND_BLACK: Format = Format::new().set_border(FormatBorder::Thin).set_background_color(Color::Black);
    pub static ref BACKGROUND_GREEN: Format = Format::new().set_border(FormatBorder::Thin).set_background_color(Color::Green).set_text_wrap().set_align(FormatAlign::Center);
    pub static ref BACKGROUND_RED: Format = Format::new().set_border(FormatBorder::Thin).set_background_color(Color::Red).set_text_wrap().set_align(FormatAlign::Center);
}

