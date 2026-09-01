//! Rendering for grind mode: problem picker, typing surface, live wpm
//! candlestick chart, and the results card.
//!
//! Layout note: there is deliberately no outer border. Nesting bordered
//! panels inside an outer frame costs two rows and four columns and adds a
//! second rule beside every inner rule, which on an 80x24 terminal is a real
//! amount of the code pane. A plain title row does the same job.

use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph};

use super::app::{GrindApp, Screen};
use super::chart::CandleChart;
use super::problems::{Difficulty, PROBLEMS};
use super::theme::theme;

pub fn render(app: &mut GrindApp, f: &mut Frame<'_, impl Backend>) {
    let size = f.size();
    match app.screen {
        Screen::Select => render_select(app, f, size),
        Screen::Typing | Screen::Results => render_run(app, f, size),
    }
}

fn difficulty_color(d: Difficulty) -> Color {
    match d {
        Difficulty::Easy => theme().bull,
        Difficulty::Medium => theme().amber,
        Difficulty::Hard => theme().bear,
    }
}

fn key_hints(pairs: &[(&str, &str)]) -> Line<'static> {
    let mut spans = vec![Span::raw("  ")];
    for (key, label) in pairs {
        spans.push(Span::styled(
            key.to_string(),
            Style::default()
                .fg(theme().ink)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}   ", label),
            Style::default().fg(theme().muted),
        ));
    }
    Line::from(spans)
}

fn render_select(app: &mut GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "  GRIND",
            Style::default()
                .fg(theme().bull)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  type the solution — your wpm is the ticker",
            Style::default().fg(theme().muted),
        )),
    ]);
    f.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = PROBLEMS
        .iter()
        .map(|p| {
            ListItem::new(Line::from(vec![
                Span::styled("● ", Style::default().fg(difficulty_color(p.difficulty))),
                Span::styled(format!("{:<46}", p.title), Style::default().fg(theme().ink)),
                Span::styled(
                    format!("{:<8}", p.difficulty.as_str().to_lowercase()),
                    Style::default().fg(difficulty_color(p.difficulty)),
                ),
                Span::styled(p.language, Style::default().fg(theme().muted)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().bg(theme().rule))
        .highlight_symbol("▌");
    let mut state = ListState::default();
    state.select(Some(app.selected));
    f.render_stateful_widget(list, chunks[1], &mut state);

    f.render_widget(
        Paragraph::new(key_hints(&[
            ("↑↓/jk", "select"),
            ("enter", "start"),
            ("q", "quit"),
        ])),
        chunks[2],
    );
}

fn render_run(app: &mut GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect) {
    let now = app.now_secs();
    // Size the code pane to the snippet rather than letting it absorb all the
    // slack: a 5-line solution does not need 19 rows, and every row it does
    // not need is a row the chart can use.
    let code_lines = app.engine.target.iter().filter(|t| t.ch == '\n').count() as u16 + 1;
    let max_code_h = area.height.saturating_sub(12).max(4);
    let mut code_h = (code_lines + 1).clamp(4, max_code_h);
    if app.screen == Screen::Results {
        // The results card takes over the code pane rather than floating over
        // the chart: once the run is done the code has served its purpose, and
        // covering the candles hides the thing the run just produced.
        code_h = code_h.max(9).min(area.height.saturating_sub(6).max(4));
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Length(1), // stat strip
            Constraint::Length(1), // progress
            Constraint::Length(code_h),
            Constraint::Min(6),    // chart takes the rest
            Constraint::Length(1), // hints
        ])
        .split(area);

    render_title(app, f, chunks[0]);
    render_stats(app, f, chunks[1], now);
    render_progress(app, f, chunks[2]);
    if app.screen == Screen::Results {
        render_results(app, f, chunks[3], now);
    } else {
        render_code(app, f, chunks[3]);
    }
    render_chart(app, f, chunks[4], now);

    let hints = if app.screen == Screen::Results {
        key_hints(&[("enter", "next"), ("^r", "retry"), ("esc", "menu")])
    } else {
        key_hints(&[("esc", "menu"), ("^r", "restart"), ("^c", "quit")])
    };
    f.render_widget(Paragraph::new(hints), chunks[5]);
}

fn render_title(app: &GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect) {
    let p = app.problem();
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                p.title.to_uppercase(),
                Style::default()
                    .fg(theme().ink)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ·  ", Style::default().fg(theme().muted)),
            Span::styled(
                p.difficulty.as_str().to_lowercase(),
                Style::default().fg(difficulty_color(p.difficulty)),
            ),
        ])),
        area,
    );
}

/// The ticker line. Direction is shown with an arrow as well as colour, so
/// it still reads without colour vision.
fn render_stats(app: &GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect, now: f64) {
    let e = &app.engine;
    let live = e.rolling_wpm(now);
    let avg = e.average_wpm(now);
    let up = live >= avg;
    let (arrow, color) = if up {
        ("▲", theme().bull)
    } else {
        ("▼", theme().bear)
    };

    let mut spans = vec![
        Span::raw("  "),
        Span::styled(
            format!("{:.0}", live),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {} ", arrow), Style::default().fg(color)),
        Span::styled("WPM", Style::default().fg(theme().muted)),
        Span::raw("    "),
    ];
    for (label, value) in [
        ("avg", format!("{:.0}", avg)),
        ("acc", format!("{:.1}%", e.accuracy())),
        ("time", format!("{:.0}s", e.elapsed_secs(now))),
    ] {
        spans.push(Span::styled(
            format!("{} ", label),
            Style::default().fg(theme().muted),
        ));
        spans.push(Span::styled(value, Style::default().fg(theme().ink)));
        spans.push(Span::raw("   "));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_progress(app: &GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect) {
    if area.width < 8 {
        return;
    }
    let pct = app.engine.progress_percent() / 100.0;
    let bar_w = area.width.saturating_sub(4) as usize;
    let filled = ((bar_w as f64) * pct).round() as usize;
    let line = Line::from(vec![
        Span::raw("  "),
        Span::styled("━".repeat(filled), Style::default().fg(theme().bull)),
        Span::styled(
            "━".repeat(bar_w.saturating_sub(filled)),
            Style::default().fg(theme().rule),
        ),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn render_code(app: &GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect) {
    let e = &app.engine;
    let mut lines: Vec<Line> = vec![];
    let mut current: Vec<Span> = vec![];
    let mut cursor_row: u16 = 0;

    for (i, tc) in e.target.iter().enumerate() {
        let at_cursor = i == e.cursor && !e.is_finished();
        let style = if at_cursor {
            if app.flash_error {
                Style::default().bg(theme().bear).fg(Color::Black)
            } else {
                Style::default().bg(theme().ink).fg(Color::Black)
            }
        } else if i < e.cursor {
            if tc.had_error {
                Style::default().fg(theme().bear)
            } else {
                Style::default().fg(theme().ink)
            }
        } else {
            Style::default().fg(theme().pending)
        };
        if at_cursor {
            cursor_row = lines.len() as u16;
        }
        if tc.ch == '\n' {
            if at_cursor {
                current.push(Span::styled("⏎", style));
            }
            lines.push(Line::from(std::mem::take(&mut current)));
        } else {
            current.push(Span::styled(tc.ch.to_string(), style));
        }
    }
    lines.push(Line::from(current));

    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(theme().rule))
        .padding(ratatui::widgets::Padding::new(1, 0, 0, 0));
    let text_h = block.inner(area).height;
    let scroll = cursor_row.saturating_sub(text_h.saturating_sub(2));
    f.render_widget(Paragraph::new(lines).block(block).scroll((scroll, 0)), area);
}

fn render_chart(app: &GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect, now: f64) {
    let candles = app.series.all();
    let last = candles.last();
    let up = last.map(|c| c.is_bullish()).unwrap_or(true);

    let title = Line::from(vec![
        Span::styled(" $WPM ", Style::default().fg(theme().ink)),
        Span::styled(
            format!("{:.0} ", app.engine.rolling_wpm(now)),
            Style::default()
                .fg(if up { theme().bull } else { theme().bear })
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("· high {:.0} · 2s ", app.series.session_high()),
            Style::default().fg(theme().muted),
        ),
    ]);

    let chart = CandleChart::new(&candles).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme().rule))
            .title(title),
    );
    f.render_widget(chart, area);
}

fn render_results(app: &GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect, now: f64) {
    let e = &app.engine;
    let candles = app.series.all();
    let bulls = candles.iter().filter(|c| c.is_bullish()).count();
    let bears = candles.len().saturating_sub(bulls);
    let up = bulls >= bears;
    let (verdict, color) = if up {
        ("theme().bullISH", theme().bull)
    } else {
        ("theme().bearISH", theme().bear)
    };

    let w = 46.min(area.width);
    let h = 9.min(area.height);
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(w)) / 2,
        y: area.y + (area.height.saturating_sub(h)) / 3,
        width: w,
        height: h,
    };
    f.render_widget(Clear, popup);

    let lines = vec![
        Line::from(Span::styled(
            "run complete",
            Style::default().fg(theme().muted),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{:.0}", e.average_wpm(now)),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" wpm     ", Style::default().fg(theme().muted)),
            Span::styled(
                format!("{:.1}%", e.accuracy()),
                Style::default().fg(theme().ink),
            ),
            Span::styled(" acc", Style::default().fg(theme().muted)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{:.1}s", e.elapsed_secs(now)),
                Style::default().fg(theme().ink),
            ),
            Span::styled("       high ", Style::default().fg(theme().muted)),
            Span::styled(
                format!("{:.0}", app.series.session_high()),
                Style::default().fg(theme().ink),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{} green / {} red  ", bulls, bears),
                Style::default().fg(theme().muted),
            ),
            Span::styled(
                verdict,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    f.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(color)),
        ),
        popup,
    );
}
