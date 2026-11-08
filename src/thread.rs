//! Lays a conversation out into the rows the thread pane draws, and picks which
//! of them are in view.
//!
//! A thread scrolls in rendered rows, never in messages or newlines. A message
//! is one to many rows once wrapped, so an offset counted in messages jumps by
//! a different height on every keypress, and an offset counted in newlines
//! drifts from what the pane actually shows. Wrapping happens here, at the
//! pane's width, and the pane draws exactly these rows with no wrapping of its
//! own, so the row count scrolled and the row count drawn are one number.
//!
//! In a group thread, a sender's name comes back over the top of the pane once
//! their own name row has scrolled off, while their run fills the rows under
//! it, as sticky scroll does in an editor. That band covers the window's first
//! row rather than pushing the rows down, so it changes neither count.

use std::ops::Range;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::conversation::{Conversation, Sender};

/// Which edge of the pane a row sits against: the other party's on the left,
/// mine on the right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Theirs,
    Mine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    /// The sender's name above a run of their messages in a group thread.
    Sender,
    /// One wrapped row of a message's text.
    Text,
    /// The blank row separating two messages.
    Gap,
    /// A sender's name drawn over the top row while their run scrolls under
    /// it, so the pane can mark it apart from the sender row it repeats.
    Band,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Index of the message this row belongs to.
    pub message: usize,
    pub kind: RowKind,
    pub side: Side,
    pub text: String,
    /// Index of the sender row this row sits under, in a group thread's run
    /// of one sender's messages. A sender row has none of its own.
    pub owner: Option<usize>,
}

/// The share of the pane a message may span, so a long message reads as a
/// bubble against its own edge rather than as a full-width paragraph.
const BUBBLE_NUMERATOR: usize = 3;
const BUBBLE_DENOMINATOR: usize = 4;

pub fn bubble_width(pane_width: usize) -> usize {
    (pane_width * BUBBLE_NUMERATOR / BUBBLE_DENOMINATOR).max(1)
}

pub fn layout(conversation: &Conversation, pane_width: usize) -> Vec<Row> {
    let width = bubble_width(pane_width);
    let mut rows = Vec::new();
    let mut previous: Option<&Sender> = None;
    // The sender row of the run being laid out.
    let mut run: Option<usize> = None;

    for (index, message) in conversation.messages.iter().enumerate() {
        let side = match message.from {
            Sender::Me => Side::Mine,
            Sender::Them(_) => Side::Theirs,
        };
        let starts_run = previous != Some(&message.from);
        // A gap between two runs belongs to neither, so the band never names
        // the run below it over the run above.
        if starts_run {
            run = None;
        }
        if index > 0 {
            rows.push(Row {
                message: index,
                kind: RowKind::Gap,
                side,
                text: String::new(),
                owner: run,
            });
        }
        if let Sender::Them(name) = &message.from
            && conversation.group
            && starts_run
        {
            rows.push(Row {
                message: index,
                kind: RowKind::Sender,
                side,
                text: name.clone(),
                owner: None,
            });
            run = Some(rows.len() - 1);
        }
        for text in wrap(&message.text, width) {
            rows.push(Row {
                message: index,
                kind: RowKind::Text,
                side,
                text,
                owner: run,
            });
        }
        previous = Some(&message.from);
    }
    rows
}

/// Breaks text into rows no wider than `width` display columns.
///
/// Words break at spaces. A word wider than a row, a URL most often, is cut at
/// the column limit rather than left to overflow. Each line break in the text
/// starts a new row, and an empty line keeps its row. Width is display columns,
/// not chars or bytes, so a CJK character or an emoji counts as the two
/// columns it occupies.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut rows = Vec::new();

    for line in text.split('\n') {
        let mut row = String::new();
        for word in line.split_whitespace() {
            let needed = if row.is_empty() {
                word.width()
            } else {
                row.width() + 1 + word.width()
            };
            if needed <= width {
                if !row.is_empty() {
                    row.push(' ');
                }
                row.push_str(word);
                continue;
            }
            if !row.is_empty() {
                rows.push(std::mem::take(&mut row));
            }
            for ch in word.chars() {
                let ch_width = ch.width().unwrap_or(0);
                if row.width() + ch_width > width && !row.is_empty() {
                    rows.push(std::mem::take(&mut row));
                }
                row.push(ch);
            }
        }
        rows.push(row);
    }
    rows
}

/// The largest scroll offset that still fills the pane, counted in rows up
/// from the newest one.
pub fn max_scroll(total: usize, height: usize) -> usize {
    total.saturating_sub(height)
}

/// The rows in view, given how many rows the pane holds and how far the reader
/// has scrolled up from the newest row. An offset past the oldest row shows
/// the oldest rows rather than an empty pane.
fn window(total: usize, height: usize, scroll: usize) -> Range<usize> {
    let end = total - scroll.min(max_scroll(total, height));
    end.saturating_sub(height)..end
}

/// The rows the pane draws, top first: the window, with the band in place of
/// its first row.
pub fn visible(rows: &[Row], height: usize, scroll: usize) -> Vec<Row> {
    let window = window(rows.len(), height, scroll);
    let mut shown = rows[window.clone()].to_vec();
    if let Some(owner) = band(rows, &window, height) {
        shown[0] = Row {
            kind: RowKind::Band,
            owner: None,
            ..rows[owner].clone()
        };
    }
    shown
}

/// The sender row the band repeats over the window's first row, once that
/// row's sender has scrolled off above it. The band covers that first row, so
/// it names whoever sent the row it leaves showing. It names nobody once that
/// row is a sender row or the gap ending a run, so a run's last row reaches the
/// top unnamed rather than lose its text under a name. It takes at most a third
/// of the pane, so a pane under three rows draws none.
fn band(rows: &[Row], window: &Range<usize>, height: usize) -> Option<usize> {
    if height < 3 {
        return None;
    }
    rows.get(window.start)?.owner?;
    rows.get(window.start + 1)?.owner
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::Message;
    use crate::fixture;

    fn conversation(group: bool, messages: Vec<Message>) -> Conversation {
        Conversation {
            id: "test".to_owned(),
            name: "test".to_owned(),
            group,
            messages,
        }
    }

    fn message(from: Sender, text: &str) -> Message {
        Message {
            from,
            text: text.to_owned(),
        }
    }

    fn them(name: &str) -> Sender {
        Sender::Them(name.to_owned())
    }

    #[test]
    fn wrap_breaks_at_spaces_within_the_width() {
        assert_eq!(wrap("one two three four", 9), ["one two", "three", "four"]);
    }

    #[test]
    fn wrap_cuts_a_word_wider_than_the_row() {
        assert_eq!(wrap("see https://example.com/a", 8), ["see", "https://", "example.", "com/a"]);
    }

    #[test]
    fn wrap_keeps_line_breaks_and_empty_lines() {
        assert_eq!(wrap("first\n\nthird", 20), ["first", "", "third"]);
    }

    #[test]
    fn wrap_counts_display_columns_not_chars() {
        // Each character is two columns wide, so four of them fill eight.
        let rows = wrap("林美玲林美玲", 8);
        assert_eq!(rows, ["林美玲林", "美玲"]);
        assert!(rows.iter().all(|row| row.width() <= 8));
    }

    // A sender row is never wrapped, and the pane clips a long name at its
    // edge, so only text rows are held to the bubble width.
    #[test]
    fn every_fixture_text_row_fits_its_bubble() {
        for pane_width in [1, 7, 20, 40, 80, 200] {
            for conversation in fixture::conversations() {
                for row in layout(&conversation, pane_width)
                    .into_iter()
                    .filter(|row| row.kind == RowKind::Text)
                {
                    assert!(
                        row.text.width() <= bubble_width(pane_width) || row.text.chars().count() == 1,
                        "{:?} at pane width {pane_width}",
                        row
                    );
                }
            }
        }
    }

    #[test]
    fn a_long_message_takes_more_rows_in_a_narrower_pane() {
        let long = "word ".repeat(40);
        let thread = conversation(false, vec![message(Sender::Me, &long)]);
        let wide = layout(&thread, 120).len();
        let narrow = layout(&thread, 40).len();
        assert!(narrow > wide, "{narrow} rows at 40 against {wide} at 120");
    }

    #[test]
    fn messages_are_separated_by_one_gap_row() {
        let thread = conversation(false, vec![message(them("Sam"), "hi"), message(Sender::Me, "hello")]);
        let kinds: Vec<_> = layout(&thread, 40).iter().map(|row| (row.message, row.kind, row.side)).collect();
        assert_eq!(
            kinds,
            [
                (0, RowKind::Text, Side::Theirs),
                (1, RowKind::Gap, Side::Mine),
                (1, RowKind::Text, Side::Mine)
            ]
        );
    }

    #[test]
    fn a_group_names_each_run_of_one_senders_messages_once() {
        let thread = conversation(
            true,
            vec![
                message(them("Ana"), "a"),
                message(them("Ana"), "b"),
                message(them("Jo"), "c"),
                message(Sender::Me, "d"),
                message(them("Jo"), "e"),
            ],
        );
        let senders: Vec<_> = layout(&thread, 40)
            .into_iter()
            .filter(|row| row.kind == RowKind::Sender)
            .map(|row| (row.message, row.text))
            .collect();
        assert_eq!(senders, [(0, "Ana".to_owned()), (2, "Jo".to_owned()), (4, "Jo".to_owned())]);
    }

    #[test]
    fn a_one_to_one_thread_names_no_sender() {
        let thread = conversation(false, vec![message(them("Sam"), "a"), message(them("Sam"), "b")]);
        assert!(layout(&thread, 40).iter().all(|row| row.kind != RowKind::Sender));
    }

    #[test]
    fn the_band_names_a_sender_only_while_their_run_fills_the_top() {
        let thread = conversation(
            true,
            vec![
                message(them("Ana"), "a1\na2\na3"),
                message(them("Ana"), "b"),
                message(them("Jo"), "c1\nc2\nc3"),
            ],
        );
        // Rows: Ana, a1, a2, a3, gap, b, gap, Jo, c1, c2, c3.
        let rows = layout(&thread, 40);
        let shown = |height, scroll| -> Vec<String> {
            visible(&rows, height, scroll)
                .iter()
                .map(|row| format!("{:?} {}", row.kind, row.text))
                .collect()
        };
        let tops: Vec<_> = (0..=8).rev().map(|scroll| shown(3, scroll)[0].clone()).collect();
        assert_eq!(
            tops,
            [
                "Sender Ana",
                "Band Ana",
                "Band Ana",
                "Band Ana",
                "Band Ana",
                "Text b",
                "Gap ",
                "Sender Jo",
                "Band Jo"
            ]
        );
        // The band covers the first row rather than pushing the rest down.
        assert_eq!(shown(3, 7), ["Band Ana", "Text a2", "Text a3"]);
        assert_eq!(shown(2, 7), ["Text a2", "Text a3"]);
    }

    #[test]
    fn the_window_counts_up_from_the_newest_row_and_stops_at_the_oldest() {
        for ((total, height, scroll), expected) in [
            ((50, 10, 0), 40..50),
            ((50, 10, 1), 39..49),
            ((50, 10, 1000), 0..10),
            ((4, 10, 3), 0..4),
        ] {
            assert_eq!(
                window(total, height, scroll),
                expected,
                "{total} rows, height {height}, scroll {scroll}"
            );
        }
    }
}
