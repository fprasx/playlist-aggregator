use std::fmt::Display;

pub const CLEAR: &str = "\x1b[2J\x1b[1;1H";

// Don't want people making more of these ;)
#[non_exhaustive]
pub struct Color(&'static str);

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

// https://stackoverflow.com/questions/5947742/how-to-change-the-output-color-of-echo-in-linux?noredirect=1&lq=1
pub const RESET: Color = Color("\x1B[0m");

pub const BLACK: Color = Color("\x1B[0;30m"); // Black
pub const RED: Color = Color("\x1B[0;31m"); // Red
pub const GREEN: Color = Color("\x1B[0;32m"); // Green
pub const YELLOW: Color = Color("\x1B[0;33m"); // Yellow
pub const BLUE: Color = Color("\x1B[0;34m"); // Blue
pub const PURPLE: Color = Color("\x1B[0;35m"); // Purple
pub const CYAN: Color = Color("\x1B[0;36m"); // Cyan
pub const WHITE: Color = Color("\x1B[0;37m"); // White

// Bold
pub const BBLACK: Color = Color("\x1B[1;30m"); // Black
pub const BRED: Color = Color("\x1B[1;31m"); // Red
pub const BGREEN: Color = Color("\x1B[1;32m"); // Green
pub const BYELLOW: Color = Color("\x1B[1;33m"); // Yellow
pub const BBLUE: Color = Color("\x1B[1;34m"); // Blue
pub const BPURPLE: Color = Color("\x1B[1;35m"); // Purple
pub const BCYAN: Color = Color("\x1B[1;36m"); // Cyan
pub const BWHITE: Color = Color("\x1B[1;37m"); // White

// Underline
pub const UBLACK: Color = Color("\x1B[4;30m"); // Black
pub const URED: Color = Color("\x1B[4;31m"); // Red
pub const UGREEN: Color = Color("\x1B[4;32m"); // Green
pub const UYELLOW: Color = Color("\x1B[4;33m"); // Yellow
pub const UBLUE: Color = Color("\x1B[4;34m"); // Blue
pub const UPURPLE: Color = Color("\x1B[4;35m"); // Purple
pub const UCYAN: Color = Color("\x1B[4;36m"); // Cyan
pub const UWHITE: Color = Color("\x1B[4;37m"); // White

// Background
pub const ON_BLACK: Color = Color("\x1B[40m"); // Black
pub const ON_RED: Color = Color("\x1B[41m"); // Red
pub const ON_GREEN: Color = Color("\x1B[42m"); // Green
pub const ON_YELLOW: Color = Color("\x1B[43m"); // Yellow
pub const ON_BLUE: Color = Color("\x1B[44m"); // Blue
pub const ON_PURPLE: Color = Color("\x1B[45m"); // Purple
pub const ON_CYAN: Color = Color("\x1B[46m"); // Cyan
pub const ON_WHITE: Color = Color("\x1B[47m"); // White

// High Intensity
pub const IBLACK: Color = Color("\x1B[0;90m"); // Black
pub const IRED: Color = Color("\x1B[0;91m"); // Red
pub const IGREEN: Color = Color("\x1B[0;92m"); // Green
pub const IYELLOW: Color = Color("\x1B[0;93m"); // Yellow
pub const IBLUE: Color = Color("\x1B[0;94m"); // Blue
pub const IPURPLE: Color = Color("\x1B[0;95m"); // Purple
pub const ICYAN: Color = Color("\x1B[0;96m"); // Cyan
pub const IWHITE: Color = Color("\x1B[0;97m"); // White

// Bold High Intensity
pub const BIBLACK: Color = Color("\x1B[1;90m"); // Black
pub const BIRED: Color = Color("\x1B[1;91m"); // Red
pub const BIGREEN: Color = Color("\x1B[1;92m"); // Green
pub const BIYELLOW: Color = Color("\x1B[1;93m"); // Yellow
pub const BIBLUE: Color = Color("\x1B[1;94m"); // Blue
pub const BIPURPLE: Color = Color("\x1B[1;95m"); // Purple
pub const BICYAN: Color = Color("\x1B[1;96m"); // Cyan
pub const BIWHITE: Color = Color("\x1B[1;97m"); // White

// High Intensity backgrounds
pub const ON_IBLACK: Color = Color("\x1B[0;100m"); // Black
pub const ON_IRED: Color = Color("\x1B[0;101m"); // Red
pub const ON_IGREEN: Color = Color("\x1B[0;102m"); // Green
pub const ON_IYELLOW: Color = Color("\x1B[0;103m"); // Yellow
pub const ON_IBLUE: Color = Color("\x1B[0;104m"); // Blue
pub const ON_IPURPLE: Color = Color("\x1B[0;105m"); // Purple
pub const ON_ICYAN: Color = Color("\x1B[0;106m"); // Cyan
pub const ON_IWHITE: Color = Color("\x1B[0;107m"); // White
