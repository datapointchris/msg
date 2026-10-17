//! Hand-made conversations for the view, covering each shape it branches on.
//! Every name and number here is invented.
//!
//! The set is chosen for those shapes rather than for realism: a thread long
//! enough to scroll, a paragraph wider than any pane, a word with no break
//! point, an explicit line break, wide characters, a group with consecutive
//! messages from one sender, a contact known only by number, a single message,
//! and a conversation with none.

use crate::conversation::{Conversation, Message, Sender};

fn me(text: &str) -> Message {
    Message {
        from: Sender::Me,
        text: text.to_owned(),
    }
}

fn them(name: &str, text: &str) -> Message {
    Message {
        from: Sender::Them(name.to_owned()),
        text: text.to_owned(),
    }
}

fn one_to_one(name: &str, messages: Vec<Message>) -> Conversation {
    Conversation {
        id: name.to_owned(),
        name: name.to_owned(),
        group: false,
        messages,
    }
}

pub fn conversations() -> Vec<Conversation> {
    let sam = "Sam Rivera";
    let ana = "Ana";
    let jo = "Jo";
    let priya = "Priya";

    vec![
        one_to_one(
            sam,
            vec![
                them(sam, "Are you around this weekend?"),
                me("Saturday yes, Sunday is the move"),
                them(sam, "Oh right, the move. Need hands?"),
                me("Always. Van is booked for 9"),
                them(sam, "I can be there by 9:30, I have to drop the dog off first"),
                me("Perfect"),
                them(sam, "How many boxes are we talking"),
                me("Honestly I stopped counting at forty"),
                them(sam, "Forty!"),
                me("Most of them are books"),
                them(sam, "Of course they are"),
                me(
                    "In my defense I gave away two whole shelves last month and it made no visible difference, which says \
                     something about how many there were to start with and nothing good about my restraint at the shop",
                ),
                them(sam, "Ha"),
                them(
                    sam,
                    "Bring the good tape this time, not the stuff that peels off the second it gets cold",
                ),
                me("Already bought a case of it"),
                them(sam, "Do you want me to bring the dolly?"),
                me("Yes please, the sofa is going to be a problem"),
                them(sam, "The sofa is always a problem"),
                me("Stairs at the new place are narrow too"),
                them(sam, "We'll take the legs off. Worked last time"),
                me("Good call"),
                them(sam, "Address?"),
                me("14 Alder Row\nFlat 3, buzzer is broken so just call"),
                them(sam, "Got it"),
                them(sam, "Pizza after?"),
                me("Pizza after, my treat"),
                them(sam, "Deal. See you Saturday"),
            ],
        ),
        Conversation {
            id: "Climbing".to_owned(),
            name: "Climbing".to_owned(),
            group: true,
            messages: vec![
                them(ana, "Wall on Thursday?"),
                them(jo, "I'm in"),
                them(jo, "Can someone lend me shoes, mine finally split"),
                me("I have a spare pair, what size?"),
                them(jo, "42"),
                me("Close enough, they're 42.5"),
                them(priya, "Thursday works but I'll be late, maybe 7:15"),
                them(ana, "No rush, we'll warm up on the slab"),
                them(priya, "Has anyone tried the new overhang set? Looked brutal from the photos"),
                them(
                    ana,
                    "Tried the first two problems. The second one has a heel hook I could not figure out at all",
                ),
                them(
                    ana,
                    "Video here: https://example.com/climbing/sessions/2026/overhang-set-problem-two-heel-hook-beta",
                ),
                me("That looks impossible"),
                them(jo, "That looks fun"),
                them(priya, "Jo has a different definition of fun"),
            ],
        },
        one_to_one(
            "+1 555 0142",
            vec![
                them(
                    "+1 555 0142",
                    "Your parcel is held at the depot. Reply 1 to rebook delivery or 2 to collect.",
                ),
                me("2"),
                them("+1 555 0142", "Thanks. It will be ready to collect from 10:00 tomorrow."),
            ],
        ),
        one_to_one(
            "林 美玲",
            vec![
                them("林 美玲", "明天见！"),
                me("See you tomorrow 🙂 I'll bring the 🍰"),
                them("林 美玲", "好的 👍👍👍"),
            ],
        ),
        one_to_one("Dentist", vec![them("Dentist", "Reminder: check-up on the 21st at 3pm.")]),
        one_to_one("Alex Chen", Vec::new()),
    ]
}
