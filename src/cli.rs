use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IconStyle {
    #[default]
    #[value(
        name = "nerd",
        help = "Nerd Font icons (requires a Nerd Font installed)"
    )]
    Nerd,
    #[value(name = "emoji", help = "Standard Unicode Emoji icons")]
    Emoji,
    #[value(
        name = "ascii",
        help = "Plain ASCII indicators ([+] for dirs, spaces for files)"
    )]
    Ascii,
}

#[derive(Parser, Debug)]
#[command(
    name = "velocity",
    author = "Velocity Developers",
    version,
    about = "Velocity: Blazingly fast terminal file browser with two-pane layout, true color, and Nerd Font icons.",
    long_about = "Velocity - A blazingly fast and lightweight terminal file browser written in Rust.\nFeatures two-pane view, real-time file preview, instant search filter, and zero dependencies."
)]
pub struct Cli {
    /// Starting directory path [default: current working directory]
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Icon display style
    #[arg(short = 'i', long = "icons", value_enum, default_value = "nerd")]
    pub icons: IconStyle,

    /// Show hidden files and directories (names starting with '.')
    #[arg(short = 'a', long = "all", default_value_t = false)]
    pub all: bool,

    /// Check for updates and automatically upgrade to the latest release
    #[arg(short = 'u', long = "update", default_value_t = false)]
    pub update: bool,

    /// Check if a newer version of Velocity is available without upgrading
    #[arg(long = "check-update", default_value_t = false)]
    pub check_update: bool,
}
