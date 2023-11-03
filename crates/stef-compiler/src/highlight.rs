use std::fmt::Display;

use owo_colors::OwoColorize;

pub fn sample(value: impl Display) -> String {
    if cfg!(feature = "debug") {
        format!("❬B❭{value}❬B❭")
    } else {
        value.bright_blue().to_string()
    }
}

pub fn value(value: impl Display) -> String {
    if cfg!(feature = "debug") {
        format!("❬Y❭{value}❬Y❭")
    } else {
        value.bright_yellow().to_string()
    }
}

pub fn focus(value: impl Display) -> String {
    if cfg!(feature = "debug") {
        format!("❬W❭{value}❬W❭")
    } else {
        value.bright_white().to_string()
    }
}
