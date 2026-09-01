use super::{notification::NotifContent, *};
use crate::app_ui::components::color::TokyoNightColors;
use crate::app_ui::{async_task_channel::ChannelRequestSender, helpers::question};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph},
};

#[derive(Debug)]
pub struct Stats {
    common_state: CommonState,
    stat_state: Option<StatState>,
}

impl Stats {
    pub(crate) fn new(id: WidgetName, task_sender: ChannelRequestSender) -> Self {
        let mut cs = CommonState::new(id, task_sender, vec![]);
        cs.is_navigable = false;
        Self {
            stat_state: None,
            common_state: cs,
        }
    }
}

impl Stats {
    fn create_block(title: &str) -> Block {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(TokyoNightColors::Comment.into()))
            .title(Span::styled(
                format!(" {title} "),
                Style::default()
                    .fg(TokyoNightColors::Foreground.into())
                    .add_modifier(Modifier::BOLD),
            ))
    }
}

super::impl_common_state!(Stats);

impl Widget for Stats {
    fn render(&mut self, rect: Rect, frame: &mut Frame<CrosstermBackend<Stderr>>) {
        let block = Self::create_block("Stats");
        let inner = block.inner(rect);
        frame.render_widget(block, rect);

        let Some(stat_state) = &self.stat_state else {
            return;
        };

        // One compact row per statistic. Previously each of these was its own
        // bordered Gauge inside the Stats block inside the app frame: three
        // nested rules deep, with the border eating most of each gauge's
        // height. A label, a count and an inline bar say the same thing in one
        // row and leave the numbers actually readable.
        for (i, (label, val, total, color)) in stat_state.rows().into_iter().enumerate() {
            let i = i as u16;
            if i >= inner.height {
                break;
            }
            let row = Rect {
                x: inner.x,
                y: inner.y + i,
                width: inner.width,
                height: 1,
            };
            frame.render_widget(
                Paragraph::new(stat_row(label, val, total, color, row.width)),
                row,
            );
        }
    }

    fn process_notification(
        &mut self,
        notification: Notification,
    ) -> AppResult<Option<Notification>> {
        if let Notification::Stats(NotifContent {
            src_wid: _,
            dest_wid: _,
            content: questions,
        }) = notification
        {
            let stats = crate::app_ui::helpers::question::Stats { qm: &questions };
            self.stat_state = Some(stats.into());
        }
        Ok(None)
    }
}

impl<'a> From<question::Stats<'a>> for StatState {
    fn from(val: question::Stats<'a>) -> Self {
        StatState {
            accepted: val.get_accepted(),
            total: val.get_total_question(),
            not_attempted: val.get_not_attempted(),
            easy: val.get_easy_count(),
            medium: val.get_medium_count(),
            hard: val.get_hard_count(),
            easy_accepted: val.get_easy_accepted(),
            medium_accepted: val.get_medium_accepted(),
            hard_accepted: val.get_hard_accepted(),
        }
    }
}

/// Build one "label  n/total  ▓▓▓░░  pct" row, degrading gracefully as the
/// pane narrows: the bar is the first thing to give up its space.
fn stat_row<'a>(label: &'a str, val: usize, total: usize, color: Color, width: u16) -> Line<'a> {
    let pct = if total != 0 {
        val as f64 / total as f64
    } else {
        0.0
    };
    let counts = format!("{val}/{total}");
    let pct_text = format!("{:>5.1}%", pct * 100.0);

    let label_w = 17usize;
    let counts_w = 11usize;
    let fixed = label_w + counts_w + pct_text.len() + 2;
    let bar_w = (width as usize).saturating_sub(fixed);
    let filled = ((bar_w as f64) * pct).round() as usize;

    let mut spans = vec![
        Span::styled(
            format!("{label:<label_w$}"),
            Style::default().fg(TokyoNightColors::Foreground.into()),
        ),
        Span::styled(
            format!("{counts:<counts_w$}"),
            Style::default().fg(TokyoNightColors::Comment.into()),
        ),
    ];
    if bar_w > 0 {
        spans.push(Span::styled(
            "\u{2501}".repeat(filled),
            Style::default().fg(color),
        ));
        spans.push(Span::styled(
            "\u{2501}".repeat(bar_w.saturating_sub(filled)),
            Style::default().fg(TokyoNightColors::Selection.into()),
        ));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled(pct_text, Style::default().fg(color)));
    Line::from(spans)
}

#[derive(Debug)]
struct StatState {
    pub accepted: usize,
    pub total: usize,
    pub not_attempted: usize,
    pub easy: usize,
    pub medium: usize,
    pub hard: usize,
    pub easy_accepted: usize,
    pub medium_accepted: usize,
    pub hard_accepted: usize,
}

impl StatState {
    /// The five figures the pane shows, in display order.
    fn rows(&self) -> Vec<(&'static str, usize, usize, Color)> {
        vec![
            (
                "Total accepted",
                self.accepted,
                self.total,
                TokyoNightColors::Purple.into(),
            ),
            (
                "Total attempted",
                self.total.saturating_sub(self.not_attempted),
                self.total,
                TokyoNightColors::Purple.into(),
            ),
            (
                "Easy",
                self.easy_accepted,
                self.easy,
                TokyoNightColors::Green.into(),
            ),
            (
                "Medium",
                self.medium_accepted,
                self.medium,
                TokyoNightColors::Yellow.into(),
            ),
            (
                "Hard",
                self.hard_accepted,
                self.hard,
                TokyoNightColors::Red.into(),
            ),
        ]
    }
}
