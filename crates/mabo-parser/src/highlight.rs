use std::fmt::Display;

use owo_colors::{OwoColorize, Stream};

pub fn sample(value: impl Display) -> String {
    if cfg!(feature = "debug") {
        format!("❬B❭{value}❬B❭")
    } else {
        value
            .if_supports_color(Stream::Stderr, OwoColorize::bright_blue)
            .to_string()
    }
}

pub fn value(value: impl Display) -> String {
    if cfg!(feature = "debug") {
        format!("❬Y❭{value}❬Y❭")
    } else {
        value
            .if_supports_color(Stream::Stderr, OwoColorize::bright_yellow)
            .to_string()
    }
}
