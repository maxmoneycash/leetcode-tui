//! Colour resolution with a 16-colour fallback.
//!
//! crossterm emits 24-bit escapes unconditionally, so a terminal that does not
//! understand them can drop the colour entirely and render the chart in the
//! default foreground — candles all one colour, which is exactly the
//! information the chart exists to convey. Terminals advertise support through
//! `COLORTERM`, so check it once and fall back to named ANSI colours when it
//! is absent.

use ratatui::style::Color;
use std::sync::OnceLock;

pub struct Theme {
    /// Rising candle.
    pub bull: Color,
    /// Falling candle.
    pub bear: Color,
    /// Primary text.
    pub ink: Color,
    /// Labels and chrome.
    pub muted: Color,
    /// Not-yet-typed code.
    pub pending: Color,
    /// Rules, gridlines, empty gauge track.
    pub rule: Color,
    /// Medium difficulty.
    pub amber: Color,
}

const TRUECOLOR: Theme = Theme {
    // Green and red leaning slightly cyan/pink respectively, which separates
    // them better than pure red/green under the common colour-vision
    // deficiencies. Direction is also stated as text, so the chart never
    // depends on colour alone.
    bull: Color::Rgb(52, 208, 88),
    bear: Color::Rgb(234, 74, 90),
    ink: Color::Rgb(222, 227, 236),
    muted: Color::Rgb(120, 126, 138),
    pending: Color::Rgb(88, 94, 107),
    rule: Color::Rgb(45, 50, 60),
    amber: Color::Rgb(226, 170, 62),
};

const ANSI16: Theme = Theme {
    bull: Color::Green,
    bear: Color::Red,
    ink: Color::White,
    muted: Color::Gray,
    pending: Color::DarkGray,
    rule: Color::DarkGray,
    amber: Color::Yellow,
};

/// Does this environment advertise 24-bit colour?
///
/// Split out from the env lookup so it can be tested directly.
fn supports_truecolor(colorterm: Option<&str>, term: Option<&str>) -> bool {
    if let Some(c) = colorterm {
        let c = c.to_ascii_lowercase();
        if c.contains("truecolor") || c.contains("24bit") {
            return true;
        }
    }
    // Some terminals advertise via TERM instead, e.g. `xterm-direct`.
    term.map(|t| t.contains("direct")).unwrap_or(false)
}

pub fn theme() -> &'static Theme {
    static THEME: OnceLock<&'static Theme> = OnceLock::new();
    THEME.get_or_init(|| {
        let colorterm = std::env::var("COLORTERM").ok();
        let term = std::env::var("TERM").ok();
        if supports_truecolor(colorterm.as_deref(), term.as_deref()) {
            &TRUECOLOR
        } else {
            &ANSI16
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_truecolor_from_colorterm() {
        assert!(supports_truecolor(Some("truecolor"), None));
        assert!(supports_truecolor(Some("24bit"), None));
        assert!(supports_truecolor(Some("TrueColor"), None));
    }

    #[test]
    fn detects_direct_color_from_term() {
        assert!(supports_truecolor(None, Some("xterm-direct")));
    }

    #[test]
    fn falls_back_without_advertisement() {
        assert!(!supports_truecolor(None, None));
        assert!(!supports_truecolor(Some(""), Some("xterm-256color")));
        // 256-colour is not 24-bit; it must not be mistaken for it
        assert!(!supports_truecolor(None, Some("screen-256color")));
    }

    #[test]
    fn fallback_keeps_candle_directions_distinct() {
        // the one property the fallback must preserve
        assert_ne!(ANSI16.bull, ANSI16.bear);
        assert_ne!(TRUECOLOR.bull, TRUECOLOR.bear);
    }
}
