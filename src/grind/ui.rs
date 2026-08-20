//! Rendering for grind mode: problem picker, typing surface, live wpm
//! candlestick chart, and the results view.

use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph};

use super::app::{GrindApp, Screen};
use super::chart::CandleChart;
use super::problems::{Difficulty, PROBLEMS};

pub fn render(app: &mut GrindApp, f: &mut Frame<'_, impl Backend>) {
    let size = f.size();
    let outer = Block::default()
        .borders(Borders::ALL)
        .title(" Leetcode TUI — Grind Mode ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let inner = outer.inner(size);
    f.render_widget(outer, size);

    match app.screen {
        Screen::Select => render_select(app, f, inner),
        Screen::Typing | Screen::Results => render_run(app, f, inner),
    }
}

fn difficulty_color(d: Difficulty) -> Color {
    match d {
        Difficulty::Easy => Color::Green,
        Difficulty::Medium => Color::Yellow,
        Difficulty::Hard => Color::Red,
    }
}

fn render_select(app: &mut GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let items: Vec<ListItem> = PROBLEMS
        .iter()
        .map(|p| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<8}", p.difficulty.as_str()),
                    Style::default().fg(difficulty_color(p.difficulty)),
                ),
                Span::raw(p.title),
                Span::styled(
                    format!("  [{}]", p.language),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Pick a problem to type "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Green)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    let mut state = ListState::default();
    state.select(Some(app.selected));
    f.render_stateful_widget(list, chunks[0], &mut state);

    let help = Paragraph::new(Line::from(vec![
        Span::styled("↑/↓ or j/k", Style::default().fg(Color::Green)),
        Span::raw(" select   "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" start   "),
        Span::styled("q/Esc", Style::default().fg(Color::Green)),
        Span::raw(" quit"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(help, chunks[1]);
}

fn render_run(app: &mut GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect) {
    let now = app.now_secs();
    let chart_height = (area.height / 3).clamp(6, 12);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(4),
            Constraint::Length(chart_height),
            Constraint::Length(1),
        ])
        .split(area);

    render_status_bar(app, f, chunks[0], now);
    render_code(app, f, chunks[1]);
    render_chart(app, f, chunks[2]);
    render_run_help(app, f, chunks[3]);

    if app.screen == Screen::Results {
        render_results_popup(app, f, area, now);
    }
}

fn render_status_bar(app: &GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect, now: f64) {
    let engine = &app.engine;
    let p = app.problem();
    let wpm_now = engine.rolling_wpm(now);
    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", p.title),
            Style::default()
                .fg(difficulty_color(p.difficulty))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  $WPM {:>5.1} ", wpm_now),
            Style::default().fg(if wpm_now >= engine.average_wpm(now) {
                Color::Green
            } else {
                Color::Red
            }),
        ),
        Span::raw(format!(
            "  avg {:>5.1}   acc {:>5.1}%   {:>4.0}s   {:>3.0}%",
            engine.average_wpm(now),
            engine.accuracy(),
            engine.elapsed_secs(now),
            engine.progress_percent(),
        )),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn render_code(app: &GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect) {
    let engine = &app.engine;
    let mut lines: Vec<Line> = vec![];
    let mut current: Vec<Span> = vec![];
    let mut cursor_row: u16 = 0;

    for (i, tc) in engine.target.iter().enumerate() {
        let at_cursor = i == engine.cursor && !engine.is_finished();
        let style = if at_cursor {
            if app.flash_error {
                Style::default().bg(Color::Red).fg(Color::White)
            } else {
                Style::default().bg(Color::Green).fg(Color::Black)
            }
        } else if i < engine.cursor {
            if tc.had_error {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            }
        } else {
            Style::default().fg(Color::DarkGray)
        };
        if at_cursor {
            cursor_row = lines.len() as u16;
        }
        if tc.ch == '\n' {
            // make the newline visible when it's the char to type
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
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" type the solution ");
    let text_height = block.inner(area).height;
    // keep the cursor line in view
    let scroll = cursor_row.saturating_sub(text_height.saturating_sub(2));
    let para = Paragraph::new(lines).block(block).scroll((scroll, 0));
    f.render_widget(para, area);
}

fn render_chart(app: &GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect) {
    let candles = app.series.all();
    let title = format!(
        " $WPM · session high {:.0} · {}s candles ",
        app.series.session_high(),
        super::app::CANDLE_PERIOD_SECS
    );
    let chart = CandleChart::new(&candles).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title),
    );
    f.render_widget(chart, area);
}

fn render_run_help(app: &GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect) {
    let mut spans = vec![
        Span::styled("Esc", Style::default().fg(Color::Green)),
        Span::raw(" menu   "),
        Span::styled("Ctrl+r", Style::default().fg(Color::Green)),
        Span::raw(" restart   "),
        Span::styled("Ctrl+c", Style::default().fg(Color::Green)),
        Span::raw(" quit"),
    ];
    if app.screen == Screen::Results {
        spans = vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" next problem   "),
            Span::styled("Ctrl+r", Style::default().fg(Color::Green)),
            Span::raw(" retry   "),
            Span::styled("Esc", Style::default().fg(Color::Green)),
            Span::raw(" menu"),
        ];
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

fn render_results_popup(app: &GrindApp, f: &mut Frame<'_, impl Backend>, area: Rect, now: f64) {
    let engine = &app.engine;
    let width = 44.min(area.width);
    let height = 8.min(area.height);
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    };
    f.render_widget(Clear, popup);

    let candles = app.series.all();
    let bulls = candles.iter().filter(|c| c.is_bullish()).count();
    let bears = candles.len() - bulls;
    let verdict = if bulls >= bears { "BULLISH" } else { "BEARISH" };
    let verdict_color = if bulls >= bears {
        Color::Green
    } else {
        Color::Red
    };

    let lines = vec![
        Line::from(Span::styled(
            "run complete",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(
            "wpm {:>6.1}    acc {:>5.1}%",
            engine.average_wpm(now),
            engine.accuracy()
        )),
        Line::from(format!(
            "time {:>5.1}s   high {:>5.0} wpm",
            engine.elapsed_secs(now),
            app.series.session_high()
        )),
        Line::from(vec![
            Span::raw(format!("{} green / {} red — session ", bulls, bears)),
            Span::styled(
                verdict,
                Style::default()
                    .fg(verdict_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    let para = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(verdict_color)),
    );
    f.render_widget(para, popup);
}
