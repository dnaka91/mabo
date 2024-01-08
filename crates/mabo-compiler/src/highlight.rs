use std::fmt::Display;

use anstream::ColorChoice;
use anstyle::{AnsiColor, Color, Style};

pub fn sample(value: impl Display) -> String {
    render(value, 'B', AnsiColor::BrightBlue)
}

pub fn value(value: impl Display) -> String {
    render(value, 'Y', AnsiColor::BrightYellow)
}

pub fn focus(value: impl Display) -> String {
    render(value, 'W', AnsiColor::BrightWhite)
}

#[inline]
fn render(value: impl Display, debug_char: char, color: AnsiColor) -> String {
    if cfg!(feature = "debug") {
        format!("❬{debug_char}❭{value}❬{debug_char}❭")
    } else if use_ansi() {
        let style = Style::new().fg_color(Some(Color::Ansi(color)));
        format!("{}{value}{}", style.render(), style.render_reset())
    } else {
        value.to_string()
    }
}

fn use_ansi() -> bool {
    match anstream::stderr().current_choice() {
        ColorChoice::Auto => anstream::stderr().is_terminal(),
        ColorChoice::AlwaysAnsi | ColorChoice::Always => true,
        ColorChoice::Never => false,
    }
}
