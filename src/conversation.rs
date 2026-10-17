//! The conversations the view renders. Nothing here knows where they came
//! from, so any data source hands the view these same values.

/// Who wrote a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sender {
    Me,
    Them(String),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub from: Sender,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Conversation {
    /// Stays the same for one conversation across reads, where its position in
    /// the list does not, so state kept per conversation is keyed on this.
    /// chat.db's `chat.guid` is this kind of value.
    pub id: String,
    pub name: String,
    /// A group thread names each sender above their messages; a one-to-one
    /// thread does not, because the conversation's name already says who.
    pub group: bool,
    pub messages: Vec<Message>,
}

impl Conversation {
    /// The first line of the newest message, which is all one list row holds.
    pub fn preview(&self) -> &str {
        self.messages.last().and_then(|message| message.text.lines().next()).unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_multi_line_message_previews_as_its_first_line() {
        let conversation = Conversation {
            id: "c".to_owned(),
            name: "Sam".to_owned(),
            group: false,
            messages: vec![Message {
                from: Sender::Me,
                text: "14 Alder Row\nFlat 3".to_owned(),
            }],
        };
        assert_eq!(conversation.preview(), "14 Alder Row");
    }
}
