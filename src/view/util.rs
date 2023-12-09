use friendly::{quantity::QVal, Quantity};
use ratatui::style::Color;

use super::bin1c::Bin1C;

pub fn level_color(v: f32) -> Color {
    if v > 0.8 {
        Color::Red
    } else if v > 0.6 {
        Color::Magenta
    } else {
        Color::Green
    }
}

pub fn fmt_bytes<Q: QVal>(bytes: Q) -> String {
    Quantity::<_, Bin1C>::new(bytes)
        .sig_figs(3)
        .space(false)
        .to_string()
}

pub fn fmt_si_val<Q: QVal>(val: Q) -> String {
    Quantity::decimal(val).sig_figs(3).space(false).to_string()
}
