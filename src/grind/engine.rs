//! Typing engine for grind mode: tracks progress through a code snippet,
//! keystroke accuracy, and words-per-minute over time.
//!
//! Time is passed in as seconds since the run started so the whole engine
//! stays deterministic and unit-testable.

/// One character of the target snippet, with per-position error memory.
#[derive(Debug, Clone)]
pub struct TargetChar {
    pub ch: char,
    /// A wrong key was pressed at this position at least once.
    pub had_error: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyOutcome {
    Advanced,
    Rejected,
    Finished,
    Ignored,
}

#[derive(Debug)]
pub struct TypingEngine {
    pub target: Vec<TargetChar>,
    /// Index of the next character to type.
    pub cursor: usize,
    pub total_keystrokes: u64,
    pub error_keystrokes: u64,
    pub started_at: Option<f64>,
    pub finished_at: Option<f64>,
    /// (time, chars_typed) samples used for the rolling wpm window.
    samples: Vec<(f64, usize)>,
}

/// Rolling window (seconds) for the instantaneous wpm readout.
const WPM_WINDOW_SECS: f64 = 4.0;

impl TypingEngine {
    pub fn new(code: &str) -> Self {
        let target = code
            .chars()
            .map(|ch| TargetChar {
                ch,
                had_error: false,
            })
            .collect();
        Self {
            target,
            cursor: 0,
            total_keystrokes: 0,
            error_keystrokes: 0,
            started_at: None,
            finished_at: None,
            samples: vec![],
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished_at.is_some()
    }

    /// Feed one typed character. `'\n'` is what Enter produces. A wrong key
    /// does not advance the cursor (strict mode) but is counted against
    /// accuracy and marks the position.
    pub fn type_char(&mut self, ch: char, now_secs: f64) -> KeyOutcome {
        if self.is_finished() || self.cursor >= self.target.len() {
            return KeyOutcome::Ignored;
        }
        if self.started_at.is_none() {
            self.started_at = Some(now_secs);
        }
        self.total_keystrokes += 1;
        let expected = self.target[self.cursor].ch;
        if ch != expected {
            self.error_keystrokes += 1;
            self.target[self.cursor].had_error = true;
            return KeyOutcome::Rejected;
        }
        self.cursor += 1;
        // Typing flow: after a newline, indentation is skipped automatically,
        // the same way code typing trainers do it.
        if expected == '\n' {
            while self.cursor < self.target.len()
                && (self.target[self.cursor].ch == ' ' || self.target[self.cursor].ch == '\t')
            {
                self.cursor += 1;
            }
        }
        self.samples.push((now_secs, self.cursor));
        if self.cursor >= self.target.len() {
            self.finished_at = Some(now_secs);
            return KeyOutcome::Finished;
        }
        KeyOutcome::Advanced
    }

    /// Average wpm over the whole run (standard 5-chars-per-word).
    pub fn average_wpm(&self, now_secs: f64) -> f64 {
        let start = match self.started_at {
            Some(s) => s,
            None => return 0.0,
        };
        let end = self.finished_at.unwrap_or(now_secs);
        let mins = (end - start) / 60.0;
        if mins <= 0.0 {
            return 0.0;
        }
        (self.cursor as f64 / 5.0) / mins
    }

    /// Instantaneous wpm over the trailing few seconds — this is what feeds
    /// the candlestick chart, so it moves like a ticker.
    pub fn rolling_wpm(&self, now_secs: f64) -> f64 {
        let start = match self.started_at {
            Some(s) => s,
            None => return 0.0,
        };
        let window_start = (now_secs - WPM_WINDOW_SECS).max(start);
        // chars typed at the start of the window
        let base = self
            .samples
            .iter()
            .rev()
            .find(|(t, _)| *t <= window_start)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        // Floor the span so the first keystrokes of a run don't divide by a
        // near-zero interval and register a absurd four-digit wpm spike.
        let span = (now_secs - window_start).max(1.0);
        let chars = self.cursor.saturating_sub(base) as f64;
        (chars / 5.0) / (span / 60.0)
    }

    /// Keystroke accuracy in percent (100 when nothing typed yet).
    pub fn accuracy(&self) -> f64 {
        if self.total_keystrokes == 0 {
            return 100.0;
        }
        100.0 * (self.total_keystrokes - self.error_keystrokes) as f64
            / self.total_keystrokes as f64
    }

    pub fn elapsed_secs(&self, now_secs: f64) -> f64 {
        match self.started_at {
            Some(s) => self.finished_at.unwrap_or(now_secs) - s,
            None => 0.0,
        }
    }

    pub fn progress_percent(&self) -> f64 {
        if self.target.is_empty() {
            return 100.0;
        }
        100.0 * self.cursor as f64 / self.target.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advances_only_on_correct_char() {
        let mut e = TypingEngine::new("ab");
        assert_eq!(e.type_char('x', 0.0), KeyOutcome::Rejected);
        assert_eq!(e.cursor, 0);
        assert!(e.target[0].had_error);
        assert_eq!(e.type_char('a', 0.1), KeyOutcome::Advanced);
        assert_eq!(e.type_char('b', 0.2), KeyOutcome::Finished);
        assert!(e.is_finished());
    }

    #[test]
    fn skips_indentation_after_newline() {
        let mut e = TypingEngine::new("a\n    b");
        assert_eq!(e.type_char('a', 0.0), KeyOutcome::Advanced);
        assert_eq!(e.type_char('\n', 0.1), KeyOutcome::Advanced);
        // cursor should now sit on 'b', not the spaces
        assert_eq!(e.target[e.cursor].ch, 'b');
        assert_eq!(e.type_char('b', 0.2), KeyOutcome::Finished);
    }

    #[test]
    fn accuracy_counts_errors() {
        let mut e = TypingEngine::new("ab");
        e.type_char('x', 0.0);
        e.type_char('a', 0.1);
        e.type_char('b', 0.2);
        assert!((e.accuracy() - 66.666).abs() < 0.1);
    }

    #[test]
    fn average_wpm_matches_definition() {
        let mut e = TypingEngine::new("hello");
        for (i, ch) in "hello".chars().enumerate() {
            e.type_char(ch, i as f64);
        }
        // 5 chars = 1 word in 4 seconds -> 15 wpm
        assert!((e.average_wpm(10.0) - 15.0).abs() < 0.01);
    }

    #[test]
    fn rolling_wpm_has_no_startup_spike() {
        let mut e = TypingEngine::new("abcdefghij");
        for (i, ch) in "abcde".chars().enumerate() {
            e.type_char(ch, i as f64 * 0.01);
        }
        // 5 chars in ~0.05s must not read as thousands of wpm — the span
        // floor keeps it at burst-speed, not divide-by-epsilon speed.
        assert!(e.rolling_wpm(0.05) <= 60.0);
    }

    #[test]
    fn rolling_wpm_uses_trailing_window() {
        let mut e = TypingEngine::new("aaaaaaaaaa");
        // 5 chars in the first second, then nothing
        for i in 0..5 {
            e.type_char('a', i as f64 * 0.2);
        }
        let early = e.rolling_wpm(1.0);
        let late = e.rolling_wpm(60.0);
        assert!(early > 0.0);
        // window has slid past all activity
        assert_eq!(late, 0.0);
    }
}
