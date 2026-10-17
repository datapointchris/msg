//! The conversation view's state, and what each key does to it.
//!
//! ratatui redraws the whole frame from this state on every tick, so nothing
//! here knows about drawing. A key changes a field; the next frame reads it.
//!
//! Focus is one of two places. On the conversation list, the arrow keys pick a
//! conversation. In a conversation, typing goes to the composition line and
//! the arrow and page keys scroll the thread, so the thread and the line share
//! that focus rather than each taking a turn of their own: a reader scrolling
//! back to check a detail can carry on typing without a key in between. Tab
//! moves between the two, and Enter and Esc move in and back out.

use std::collections::HashMap;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use unicode_segmentation::UnicodeSegmentation;

use crate::conversation::Conversation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List,
    Conversation,
}

#[derive(Debug)]
pub struct App {
    pub conversations: Vec<Conversation>,
    /// The selected conversation's id rather than its row, so the selection
    /// stays on one conversation when a new message reorders the list.
    selected: Option<String>,
    pub focus: Focus,
    /// The conversations are invented, which the view labels on screen.
    pub sample: bool,
    /// One unsent draft per conversation id, so text typed for one person never
    /// follows the reader into another's thread, whatever row either sits on.
    drafts: HashMap<String, String>,
    /// Rows between the newest row and the bottom of the thread pane. Zero
    /// keeps the newest message in view, which is where a thread opens.
    pub scroll: usize,
    /// How many rows the thread pane held on the last frame, which is how far
    /// a page key moves. Only the frame knows the pane's height.
    pub page: usize,
    pub quit: bool,
}

impl App {
    pub fn new(conversations: Vec<Conversation>, sample: bool) -> Self {
        let selected = conversations.first().map(|conversation| conversation.id.clone());
        Self {
            conversations,
            selected,
            focus: Focus::List,
            sample,
            drafts: HashMap::new(),
            scroll: 0,
            page: 1,
            quit: false,
        }
    }

    /// The selected conversation's row in the list.
    pub fn selected_index(&self) -> Option<usize> {
        let id = self.selected.as_ref()?;
        self.conversations.iter().position(|conversation| &conversation.id == id)
    }

    pub fn conversation(&self) -> Option<&Conversation> {
        self.conversations.get(self.selected_index()?)
    }

    pub fn draft(&self) -> &str {
        self.selected.as_ref().and_then(|id| self.drafts.get(id)).map_or("", String::as_str)
    }

    fn draft_mut(&mut self) -> Option<&mut String> {
        let id = self.selected.clone()?;
        Some(self.drafts.entry(id).or_default())
    }

    /// Appends pasted text to the draft. Line breaks are kept, since a message
    /// can hold them.
    pub fn paste(&mut self, text: &str) {
        if self.focus != Focus::Conversation {
            return;
        }
        let text: String = text
            .replace("\r\n", "\n")
            .chars()
            .map(|ch| match ch {
                '\r' => '\n',
                '\t' => ' ',
                ch => ch,
            })
            .filter(|ch| *ch == '\n' || !ch.is_control())
            .collect();
        if let Some(draft) = self.draft_mut() {
            draft.push_str(&text);
        }
    }

    /// Records the thread pane's size from the frame being drawn, and pulls a
    /// scroll offset past the oldest row back to it. Without the clamp, a
    /// reader who held Page Up past the top would press Page Down several
    /// times before the pane moved.
    pub fn fit(&mut self, total_rows: usize, height: usize) {
        self.page = height.max(1);
        self.scroll = self.scroll.min(crate::thread::max_scroll(total_rows, height));
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        if matches!(key.code, KeyCode::Tab | KeyCode::BackTab) {
            self.focus = match self.focus {
                Focus::List => Focus::Conversation,
                Focus::Conversation => Focus::List,
            };
            return;
        }
        match self.focus {
            Focus::List => self.list_key(key),
            Focus::Conversation => self.conversation_key(key),
        }
    }

    fn list_key(&mut self, key: KeyEvent) {
        let current = self.selected_index().unwrap_or(0);
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.select(current.saturating_sub(1)),
            KeyCode::Down | KeyCode::Char('j') => self.select(current + 1),
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => self.focus = Focus::Conversation,
            KeyCode::Esc | KeyCode::Char('q') => self.quit = true,
            _ => {}
        }
    }

    fn conversation_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.focus = Focus::List,
            KeyCode::Up => self.scroll = self.scroll.saturating_add(1),
            KeyCode::Down => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::PageUp => self.scroll = self.scroll.saturating_add(self.page),
            KeyCode::PageDown => self.scroll = self.scroll.saturating_sub(self.page),
            // The next frame's fit pulls this back to the oldest row.
            KeyCode::Home => self.scroll = usize::MAX,
            KeyCode::End => self.scroll = 0,
            // One visible character, which a flag or a skin-toned emoji spans
            // several chars of.
            KeyCode::Backspace => {
                if let Some(draft) = self.draft_mut()
                    && let Some((start, _)) = draft.grapheme_indices(true).next_back()
                {
                    draft.truncate(start);
                }
            }
            // A modified key is a shortcut, not text.
            KeyCode::Char(ch) if !key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
                if let Some(draft) = self.draft_mut() {
                    draft.push(ch);
                }
            }
            // Enter would send. There is no send path, so the draft stays put.
            _ => {}
        }
    }

    fn select(&mut self, index: usize) {
        let last = self.conversations.len().saturating_sub(1);
        let id = self.conversations.get(index.min(last)).map(|conversation| conversation.id.clone());
        if id != self.selected {
            self.selected = id;
            self.scroll = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    fn app() -> App {
        App::new(fixture::conversations(), true)
    }

    fn press(app: &mut App, code: KeyCode) {
        app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
    }

    fn type_text(app: &mut App, text: &str) {
        for ch in text.chars() {
            press(app, KeyCode::Char(ch));
        }
    }

    #[test]
    fn the_list_moves_between_conversations_and_stops_at_each_end() {
        let mut app = app();
        let last = app.conversations.len() - 1;
        press(&mut app, KeyCode::Up);
        assert_eq!(app.selected_index(), Some(0));
        press(&mut app, KeyCode::Down);
        press(&mut app, KeyCode::Char('j'));
        assert_eq!(app.selected_index(), Some(2));
        for _ in 0..app.conversations.len() + 3 {
            press(&mut app, KeyCode::Down);
        }
        assert_eq!(app.selected_index(), Some(last));
        press(&mut app, KeyCode::Char('k'));
        assert_eq!(app.selected_index(), Some(last - 1));
    }

    #[test]
    fn tab_moves_focus_between_the_list_and_the_conversation() {
        let mut app = app();
        press(&mut app, KeyCode::Tab);
        assert_eq!(app.focus, Focus::Conversation);
        press(&mut app, KeyCode::Tab);
        assert_eq!(app.focus, Focus::List);
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.focus, Focus::Conversation);
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.focus, Focus::List);
        assert!(!app.quit);
    }

    #[test]
    fn typing_in_a_conversation_fills_its_composition_line() {
        let mut app = app();
        press(&mut app, KeyCode::Tab);
        type_text(&mut app, "quick j k 🇯🇵");
        press(&mut app, KeyCode::Backspace);
        assert_eq!(app.draft(), "quick j k ");
        // List keys are text here, and neither moves the selection nor quits.
        assert_eq!(app.selected_index(), Some(0));
        assert!(!app.quit);
    }

    #[test]
    fn a_pasted_table_row_or_address_lands_in_the_draft_as_text() {
        let mut app = app();
        press(&mut app, KeyCode::Tab);
        app.paste("name\tqty\r\n14 Alder Row\nFlat 3");
        assert_eq!(app.draft(), "name qty\n14 Alder Row\nFlat 3");
        assert_eq!(app.focus, Focus::Conversation);
        assert!(!app.quit);
    }

    #[test]
    fn a_draft_belongs_to_its_conversation() {
        let mut app = app();
        press(&mut app, KeyCode::Tab);
        type_text(&mut app, "for sam");
        press(&mut app, KeyCode::Tab);
        press(&mut app, KeyCode::Down);
        assert_eq!(app.draft(), "");
        press(&mut app, KeyCode::Up);
        assert_eq!(app.draft(), "for sam");
    }

    #[test]
    fn a_modified_key_types_nothing() {
        let mut app = app();
        press(&mut app, KeyCode::Tab);
        app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT));
        app.handle_key(KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT));
        assert_eq!(app.draft(), "X");
    }

    #[test]
    fn control_c_quits_from_either_focus() {
        for focus_keys in [0, 1] {
            let mut app = app();
            for _ in 0..focus_keys {
                press(&mut app, KeyCode::Tab);
            }
            app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
            assert!(app.quit);
            assert_eq!(app.draft(), "");
        }
    }

    #[test]
    fn q_quits_from_the_list() {
        let mut app = app();
        press(&mut app, KeyCode::Char('q'));
        assert!(app.quit);
    }

    #[test]
    fn the_thread_scrolls_by_row_and_by_page() {
        let mut app = app();
        press(&mut app, KeyCode::Tab);
        app.fit(100, 10);
        press(&mut app, KeyCode::Up);
        press(&mut app, KeyCode::Up);
        assert_eq!(app.scroll, 2);
        press(&mut app, KeyCode::PageUp);
        assert_eq!(app.scroll, 12);
        press(&mut app, KeyCode::Down);
        press(&mut app, KeyCode::PageDown);
        assert_eq!(app.scroll, 1);
        press(&mut app, KeyCode::PageDown);
        assert_eq!(app.scroll, 0);
    }

    #[test]
    fn scrolling_past_the_top_comes_back_on_the_first_key_down() {
        let mut app = app();
        press(&mut app, KeyCode::Tab);
        press(&mut app, KeyCode::Home);
        app.fit(30, 10);
        assert_eq!(app.scroll, 20);
        press(&mut app, KeyCode::Down);
        assert_eq!(app.scroll, 19);
    }

    #[test]
    fn changing_conversation_opens_it_at_the_newest_message() {
        let mut app = app();
        press(&mut app, KeyCode::Tab);
        app.fit(100, 10);
        press(&mut app, KeyCode::PageUp);
        press(&mut app, KeyCode::Tab);
        press(&mut app, KeyCode::Down);
        assert_eq!(app.scroll, 0);
    }
}
