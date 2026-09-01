use std::collections::HashMap;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::components::color::TokyoNightColors;

use super::{
    app::App,
    widgets::{notification::WidgetName, CrosstermStderr, Widget},
};

/// Renders the user interface widgets.
pub fn render(app: &mut App, f: &mut CrosstermStderr) {
    // grind mode takes over the full frame while it is open
    if let Some(grind) = app.grind.as_mut() {
        crate::grind::ui::render(grind, f);
        return;
    }

    // Create two chunks with equal horizontal screen space
    let size = f.size();

    // A header row rather than a full frame around the whole app: an outer
    // border costs two rows and four columns and puts a second rule beside
    // every panel rule, which on a small terminal is real estate the question
    // list needs more than the frame does.
    let shell = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " leetcode",
                Style::default()
                    .fg(TokyoNightColors::Pink.into())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " tui",
                Style::default().fg(TokyoNightColors::Comment.into()),
            ),
        ]))
        .alignment(Alignment::Left),
        shell[0],
    );

    let inner_size = shell[1];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(inner_size);

    // Stats is five fixed rows, so give it exactly what it needs and let the
    // topic list have the rest rather than splitting the column in half.
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(7)])
        .split(chunks[0]);

    let right_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(chunks[1]);

    let layout_map = HashMap::from([
        (WidgetName::TopicList, left_chunks[0]),    // tags
        (WidgetName::Stats, left_chunks[1]),        // stats
        (WidgetName::QuestionList, right_chunk[0]), // question
        (WidgetName::HelpLine, size),
    ]);

    for (name, wid) in app.widget_map.iter_mut() {
        let rect = layout_map.get(name).unwrap();
        wid.render(*rect, f);
    }

    if let Some(popup) = app.get_current_popup_mut() {
        popup.render(size, f)
    }
}
