//! State machine + event loop for grind mode. Runs standalone: no leetcode
//! session, no database — just the terminal.

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;

use crate::errors::AppResult;

use super::candles::CandleSeries;
use super::engine::{KeyOutcome, TypingEngine};
use super::problems::{GrindProblem, PROBLEMS};

/// Seconds of typing aggregated into one candle.
pub const CANDLE_PERIOD_SECS: f64 = 2.0;
/// How long the cursor stays red after a wrong key.
const ERROR_FLASH: Duration = Duration::from_millis(150);
const TICK: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Select,
    Typing,
    Results,
}

#[derive(Debug)]
pub struct GrindApp {
    pub screen: Screen,
    pub selected: usize,
    pub engine: TypingEngine,
    pub series: CandleSeries,
    pub flash_error: bool,
    pub running: bool,
    started: Instant,
    last_error_at: Option<Instant>,
}

impl Default for GrindApp {
    fn default() -> Self {
        Self::new()
    }
}

impl GrindApp {
    pub fn new() -> Self {
        Self {
            screen: Screen::Select,
            selected: 0,
            engine: TypingEngine::new(PROBLEMS[0].code),
            series: CandleSeries::new(CANDLE_PERIOD_SECS),
            flash_error: false,
            running: true,
            started: Instant::now(),
            last_error_at: None,
        }
    }

    pub fn problem(&self) -> &'static GrindProblem {
        &PROBLEMS[self.selected]
    }

    /// Wall-clock seconds since this app started; the engine and candle
    /// series both work in this time base.
    pub fn now_secs(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }

    fn start_run(&mut self) {
        self.engine = TypingEngine::new(self.problem().code);
        self.series = CandleSeries::new(CANDLE_PERIOD_SECS);
        self.flash_error = false;
        self.last_error_at = None;
        self.screen = Screen::Typing;
    }

    /// Advance error-flash timing and sample the wpm ticker.
    ///
    /// Public so the main TUI can drive grind mode from its own event loop
    /// instead of grind owning stdin.
    pub fn tick(&mut self) {
        if let Some(t) = self.last_error_at {
            if t.elapsed() >= ERROR_FLASH {
                self.flash_error = false;
                self.last_error_at = None;
            }
        }
        // Sample the ticker only while a run is live.
        if self.screen == Screen::Typing && self.engine.started_at.is_some() {
            let now = self.now_secs();
            let started = self.engine.started_at.unwrap_or(now);
            self.series
                .push_sample(now - started, self.engine.rolling_wpm(now));
        }
    }

    fn type_char(&mut self, ch: char) {
        let outcome = self.engine.type_char(ch, self.now_secs());
        match outcome {
            KeyOutcome::Rejected => {
                self.flash_error = true;
                self.last_error_at = Some(Instant::now());
            }
            KeyOutcome::Finished => {
                self.screen = Screen::Results;
            }
            _ => {}
        }
    }

    /// Feed one key event. Sets `running = false` when the user leaves the
    /// problem picker, which the host uses as the signal to close the overlay.
    pub fn handle_key(&mut self, key: event::KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && key.code == KeyCode::Char('c') {
            self.running = false;
            return;
        }
        match self.screen {
            Screen::Select => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.running = false,
                KeyCode::Up | KeyCode::Char('k') => {
                    self.selected = self.selected.checked_sub(1).unwrap_or(PROBLEMS.len() - 1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.selected = (self.selected + 1) % PROBLEMS.len();
                }
                KeyCode::Enter => self.start_run(),
                _ => {}
            },
            Screen::Typing => match key.code {
                KeyCode::Esc => self.screen = Screen::Select,
                KeyCode::Char('r') if ctrl => self.start_run(),
                KeyCode::Enter => self.type_char('\n'),
                KeyCode::Tab => self.type_char('\t'),
                KeyCode::Char(c) if !ctrl => self.type_char(c),
                _ => {}
            },
            Screen::Results => match key.code {
                KeyCode::Esc => self.screen = Screen::Select,
                KeyCode::Char('r') if ctrl => self.start_run(),
                KeyCode::Enter => {
                    self.selected = (self.selected + 1) % PROBLEMS.len();
                    self.start_run();
                }
                _ => {}
            },
        }
    }
}

/// Entry point: `leetui grind`.
pub fn run() -> AppResult<()> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    crossterm::execute!(stderr, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal);

    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<io::Stderr>>) -> AppResult<()> {
    let mut app = GrindApp::new();
    while app.running {
        app.tick();
        terminal.draw(|f| super::ui::render(&mut app, f))?;
        if event::poll(TICK)? {
            match event::read()? {
                Event::Key(key) => app.handle_key(key),
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }
    Ok(())
}
