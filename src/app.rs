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

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::fixture::Conversation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List,
    Conversation,
}

#[derive(Debug)]
pub struct App {
    pub conversations: Vec<Conversation>,
    pub selected: usize,
    pub focus: Focus,
    /// One unsent draft per conversation, so text typed for one person never
    /// follows the reader into another's thread.
    drafts: Vec<String>,
    /// Rows between the newest row and the bottom of the thread pane. Zero
    /// keeps the newest message in view, which is where a thread opens.
    pub scroll: usize,
    /// How many rows the thread pane held on the last frame, which is how far
    /// a page key moves. Only the frame knows the pane's height.
    pub page: usize,
    pub quit: bool,
}

impl App {
    pub fn new(conversations: Vec<Conversation>) -> Self {
        let drafts = vec![String::new(); conversations.len()];
        Self {
            conversations,
            selected: 0,
            focus: Focus::List,
            drafts,
            scroll: 0,
            page: 1,
            quit: false,
        }
    }

    pub fn conversation(&self) -> Option<&Conversation> {
        self.conversations.get(self.selected)
    }

    pub fn draft(&self) -> &str {
        self.drafts.get(self.selected).map_or("", String::as_str)
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
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.select(self.selected.saturating_sub(1)),
            KeyCode::Down | KeyCode::Char('j') => self.select(self.selected + 1),
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
            KeyCode::Backspace => {
                if let Some(draft) = self.drafts.get_mut(self.selected) {
                    draft.pop();
                }
            }
            // A modified key is a shortcut, not text.
            KeyCode::Char(ch) if !key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
                if let Some(draft) = self.drafts.get_mut(self.selected) {
                    draft.push(ch);
                }
            }
            // Enter would send. There is no send path, so the draft stays put.
            _ => {}
        }
    }

    fn select(&mut self, index: usize) {
        let last = self.conversations.len().saturating_sub(1);
        let index = index.min(last);
        if index != self.selected {
            self.selected = index;
            self.scroll = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    fn app() -> App {
        App::new(fixture::conversations())
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
    fn the_view_opens_on_the_first_conversation_in_the_list() {
        let app = app();
        assert_eq!((app.selected, app.focus, app.scroll), (0, Focus::List, 0));
    }

    #[test]
    fn the_list_moves_between_conversations_and_stops_at_each_end() {
        let mut app = app();
        let last = app.conversations.len() - 1;
        press(&mut app, KeyCode::Up);
        assert_eq!(app.selected, 0);
        press(&mut app, KeyCode::Down);
        press(&mut app, KeyCode::Char('j'));
        assert_eq!(app.selected, 2);
        for _ in 0..app.conversations.len() + 3 {
            press(&mut app, KeyCode::Down);
        }
        assert_eq!(app.selected, last);
        press(&mut app, KeyCode::Char('k'));
        assert_eq!(app.selected, last - 1);
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
        type_text(&mut app, "quick j k");
        press(&mut app, KeyCode::Backspace);
        assert_eq!(app.draft(), "quick j ");
        // List keys are text here, and neither moves the selection nor quits.
        assert_eq!(app.selected, 0);
        assert!(!app.quit);
    }

    #[test]
    fn enter_in_a_conversation_keeps_the_draft() {
        let mut app = app();
        press(&mut app, KeyCode::Tab);
        type_text(&mut app, "hello");
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.draft(), "hello");
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

    #[test]
    fn a_key_release_does_nothing() {
        let mut app = app();
        let mut key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        key.kind = KeyEventKind::Release;
        app.handle_key(key);
        assert_eq!(app.selected, 0);
    }
}
