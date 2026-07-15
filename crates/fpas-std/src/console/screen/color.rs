use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::console) enum RenderColor {
    Crt(u8),
    Rgb { r: u8, g: u8, b: u8 },
    Ansi256(u8),
}

const CRT_FG_SGR: [&str; 16] = [
    "\x1b[30m", "\x1b[34m", "\x1b[32m", "\x1b[36m", "\x1b[31m", "\x1b[35m", "\x1b[33m", "\x1b[37m",
    "\x1b[90m", "\x1b[94m", "\x1b[92m", "\x1b[96m", "\x1b[91m", "\x1b[95m", "\x1b[93m", "\x1b[97m",
];

const CRT_BG_SGR: [&str; 16] = [
    "\x1b[40m",
    "\x1b[44m",
    "\x1b[42m",
    "\x1b[46m",
    "\x1b[41m",
    "\x1b[45m",
    "\x1b[43m",
    "\x1b[47m",
    "\x1b[100m",
    "\x1b[104m",
    "\x1b[102m",
    "\x1b[106m",
    "\x1b[101m",
    "\x1b[105m",
    "\x1b[103m",
    "\x1b[107m",
];

impl RenderColor {
    pub(in crate::console) fn ansi_set_fg(self) -> Cow<'static, str> {
        match self {
            Self::Crt(index) => Cow::Borrowed(
                CRT_FG_SGR
                    .get(usize::from(index))
                    .copied()
                    .unwrap_or("\x1b[97m"),
            ),
            Self::Rgb { r, g, b } => Cow::Owned(format!("\x1b[38;2;{r};{g};{b}m")),
            Self::Ansi256(index) => Cow::Owned(format!("\x1b[38;5;{index}m")),
        }
    }

    pub(in crate::console) fn ansi_set_bg(self) -> Cow<'static, str> {
        match self {
            Self::Crt(index) => Cow::Borrowed(
                CRT_BG_SGR
                    .get(usize::from(index))
                    .copied()
                    .unwrap_or("\x1b[107m"),
            ),
            Self::Rgb { r, g, b } => Cow::Owned(format!("\x1b[48;2;{r};{g};{b}m")),
            Self::Ansi256(index) => Cow::Owned(format!("\x1b[48;5;{index}m")),
        }
    }

    pub(super) fn packed_index(self) -> Option<u8> {
        match self {
            Self::Crt(index) => Some(index),
            Self::Rgb { .. } | Self::Ansi256(_) => None,
        }
    }

    #[cfg(test)]
    pub(super) fn debug_label(self) -> String {
        match self {
            Self::Crt(index) => format!("crt:{index}"),
            Self::Rgb { r, g, b } => format!("rgb:{r},{g},{b}"),
            Self::Ansi256(index) => format!("ansi256:{index}"),
        }
    }
}
