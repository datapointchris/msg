//! Draws one frame from the app's state: the conversation list on the left,
//! the thread on the right, and the composition line under the thread.
//!
//! Colors are the terminal's own sixteen, so the view follows whatever theme
//! the terminal has rather than fixing a palette that is wrong in half of them.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};
use unicode_width::UnicodeWidthChar;

use crate::app::{App, Focus};
use crate::thread::{self, RowKind, Side};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [list_area, right] = Layout::horizontal([Constraint::Percentage(30), Constraint::Min(0)]).areas(frame.area());
    let [thread_area, compose_area] = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(right);

    draw_list(frame, app, list_area);
    draw_thread(frame, app, thread_area);
    draw_compose(frame, app, compose_area);
}

fn pane(title: &str, focused: bool) -> Block<'_> {
    let border = if focused {
        Style::new().fg(Color::Blue)
    } else {
        Style::new().fg(Color::DarkGray)
    };
    Block::bordered().title(title).border_style(border)
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .conversations
        .iter()
        .map(|conversation| {
            ListItem::new(vec![
                Line::from(Span::styled(conversation.name.as_str(), Style::new().add_modifier(Modifier::BOLD))),
                Line::from(Span::styled(conversation.preview(), Style::new().fg(Color::DarkGray))),
            ])
        })
        .collect();
    let title = if app.sample {
        "Conversations (sample data)"
    } else {
        "Conversations"
    };
    let list = List::new(items)
        .block(pane(title, app.focus == Focus::List))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
    let mut state = ListState::default().with_selected(app.selected_index());
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_thread(frame: &mut Frame, app: &mut App, area: Rect) {
    let Some(conversation) = app.conversation() else {
        frame.render_widget(pane("", false), area);
        return;
    };
    let block = pane(&conversation.name, app.focus == Focus::Conversation);
    let inner = block.inner(area);
    let rows = thread::layout(conversation, usize::from(inner.width));
    let height = usize::from(inner.height);

    let lines: Vec<Line> = rows[thread::window(rows.len(), height, app.scroll)]
        .iter()
        .map(|row| {
            let style = match (row.kind, row.side) {
                (RowKind::Sender, _) => Style::new().fg(Color::DarkGray),
                (_, Side::Mine) => Style::new().fg(Color::Blue),
                (_, Side::Theirs) => Style::new(),
            };
            let line = Line::styled(row.text.as_str(), style);
            match row.side {
                Side::Mine => line.right_aligned(),
                Side::Theirs => line.left_aligned(),
            }
        })
        .collect();
    let total = rows.len();
    frame.render_widget(Paragraph::new(lines).block(block), area);
    app.fit(total, height);
}

fn draw_compose(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Conversation;
    let block = pane("Message", focused);
    let inner = block.inner(area);
    // The line is one row, so a line break in the draft shows as a mark.
    let draft = app.draft().replace('\n', "↵");
    let view = compose_view(&draft, usize::from(inner.width));
    frame.render_widget(Paragraph::new(&draft[view.start..]).block(block), area);
    if focused {
        let column = u16::try_from(view.cursor).unwrap_or(u16::MAX);
        frame.set_cursor_position(Position::new(inner.x.saturating_add(column), inner.y));
    }
}

/// Which part of a draft the one-row composition line shows.
#[derive(Debug, PartialEq, Eq)]
struct ComposeView {
    /// Byte offset of the first char shown.
    start: usize,
    /// Display column of the cursor, which sits after the last char.
    cursor: usize,
}

/// Keeps the end of the draft and the cursor after it in view. A draft wider
/// than the line scrolls its start out of the left edge, the way a shell
/// prompt does, so the char just typed is always on screen.
fn compose_view(draft: &str, width: usize) -> ComposeView {
    // One column is kept for the cursor itself.
    let room = width.saturating_sub(1);
    let mut start = draft.len();
    let mut cursor = 0;
    for (index, ch) in draft.char_indices().rev() {
        let ch_width = ch.width().unwrap_or(0);
        if cursor + ch_width > room {
            break;
        }
        cursor += ch_width;
        start = index;
    }
    ComposeView { start, cursor }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_draft_that_fits_shows_whole_with_the_cursor_after_it() {
        assert_eq!(compose_view("hello", 20), ComposeView { start: 0, cursor: 5 });
    }

    #[test]
    fn a_draft_wider_than_the_line_keeps_its_end_in_view() {
        let draft = "abcdefghij";
        let view = compose_view(draft, 5);
        assert_eq!(&draft[view.start..], "ghij");
        assert_eq!(view.cursor, 4);
    }

    #[test]
    fn the_composition_line_scrolls_by_display_column() {
        let draft = "ab林美玲";
        let view = compose_view(draft, 6);
        assert_eq!(&draft[view.start..], "美玲");
        assert_eq!(view.cursor, 4);
    }
}
