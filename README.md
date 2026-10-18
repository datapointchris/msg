# msg

A terminal iMessage client, for a Linux terminal and a Mac that stays on. It is designed to read
the Mac's `chat.db` over SSH and send through Shortcuts. The screen is laid out like Messages: the
conversation list on the left, the thread on the right, and a composition line under the thread.

msg is a client of Messages on the Mac, not a replacement for it. The Mac keeps the iMessage
account and does the sending. msg's part is reading the Messages database and asking the Mac to send.

**The view works, and the channels are not wired.** Running `msg` opens the conversation view on
built-in sample conversations. It reads nothing from the Mac and sends nothing. What it is *for* is
settled and measured. Reading and sending are not written yet.

It is also its author's first Rust project built to a full set of conventions rather than as a
one-off, with the release shape, the lint posture and the toolchain pin included.

## Install

Every release ships a prebuilt `x86_64-unknown-linux-gnu` tarball and its `.sha256`.

```bash
# the newest release, prebuilt
cargo binstall msg --git https://github.com/datapointchris/msg \
  --version "$(gh release view --repo datapointchris/msg --json tagName --jq '.tagName[1:]')"

# or the tarball itself, which needs no cargo
gh release download --repo datapointchris/msg --pattern '*.tar.gz*'
sha256sum -c msg-*.sha256 && tar -xzf msg-*.tar.gz
```

**The `--version` is not optional, and the design is why.** `Cargo.toml` carries `0.0.0`
permanently, because the tag is the version. The crate is not on crates.io, so `cargo binstall` has
nowhere to learn the latest number from. Given a version, it fetches the release asset. Without
one, it reads `0.0.0` from the checkout, finds no such release, and silently compiles from source.
`--version <x.y.z>` pins one release by hand.

msg is not published to crates.io. The name there belongs to an unrelated XSI message-queue
binding, and a personal tool gains nothing from the registry but a name to defend.

## Starting from scratch

With msg installed:

```bash
msg --version   # the release's tag; a build from source prints 0.0.0
msg             # opens the view, with the list titled "Conversations (sample data)"
```

Inside the view, `Tab` moves into the selected conversation and `Esc` comes back out. `q` quits
from the list, and `Ctrl-C` quits from anywhere.

Nothing here needs a Mac or a secret yet. The sample conversations are compiled in.

The view refuses to start without a terminal on stdout. `msg | cat` exits 1 with `the conversation
view needs a terminal on stdout`, and writes nothing to stdout. stdout is data, and a screenful of
escape codes is noise to whatever reads the pipe.

## The view — the thread and the draft share one focus, so scrolling back never stops typing

| Key | In the list | In a conversation |
| --- | --- | --- |
| `Tab` `Shift-Tab` | move to the conversation | move to the list |
| `↑` `↓` | pick a conversation (`k` `j` too) | scroll the thread one row |
| `PgUp` `PgDn` | | scroll the thread one page |
| `Home` `End` | | oldest and newest message |
| `Enter` | open the conversation (`→` `l` too) | nothing yet, there is no send |
| `Backspace` | | delete one visible character, a flag or skin-toned emoji included |
| `Esc` | quit | back to the list |
| `q` | quit | types `q` |
| `Ctrl-C` | quit | quit |

In a conversation, the arrow and page keys scroll the thread and every other key types. The thread
and the composition line share that focus rather than taking turns. A reader scrolling back to
check a detail carries on typing without a key in between. A `Ctrl` or `Alt` chord types nothing.

Each conversation keeps its own draft. The draft and the selection follow the conversation, not its
row in the list. A list that reorders never carries text typed for one person into another's
thread.

A paste lands in the draft whole, line breaks included, and never runs as keys. Without bracketed
paste, a pasted tab would move focus and the text after it would run as list keys, `q` included.
The composition line is one row, so it shows a line break as `↵`.

The colors are the terminal's own sixteen, so the view follows the terminal's theme.

## The Mac — two channels over plain SSH, and AppleScript is not one of them

Both channels are verified working on macOS 15.7.7. Neither is wired into msg yet.

| Direction | Mechanism |
| --- | --- |
| Read | `sqlite3 -readonly ~/Library/Messages/chat.db` over plain SSH |
| Send | `shortcuts run <name> -i <file>` over plain SSH |

Reading works because sshd already holds Full Disk Access. Sending runs unattended, with no GUI
session and no sudo. Importing the shortcut is the one step that needs a person. macOS asks for a
click before any remote process can install an automation that sends, and nothing gets around it.

**AppleScript is not an option and should not be retried.** On 15.7.7, `send` compiles against the
live dictionary, hangs, and transmits nothing. Enumeration is broken the same way: `count of chats`
hangs, and `buddies` has left the dictionary. Conversation discovery has to come from SQLite
regardless.

## `attributedBody` — decoding it is the read path, not a fallback

**92% of messages have no readable `text` column.** On the machine it was measured against, 759 of
9814 messages carry usable text. The rest exist only as `attributedBody`, a serialized
NSAttributedString typedstream. It is not a plist, and it is not reachable from SQL. Decoding it is
the core of the project rather than a detail of it.

The decoder is [`imessage-database`](https://crates.io/crates/imessage-database). How much of it msg
can use depends on where `chat.db` is read, and that is the open decision:

| Read from | Reaches |
| --- | --- |
| a local copy of `chat.db` | the rich layer: messages, attachments, reactions, threads, edited messages |
| the live database, queried over SSH | the `attributedBody` decoder alone, with the schema work left to msg |

The rich layer opens a connection to a file on disk, so a remote query cannot feed it. A local copy
is a snapshot.

## Building from source — the tag is the version, so a source build says 0.0.0

```bash
cargo build    # rust-toolchain.toml pins the release; rustup fetches it on first use
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt
```

`Cargo.toml` carries `version = "0.0.0"` deliberately, and it is never bumped in git. The release
build checks out the tag and sets the version in that working tree before building. Only a released
binary reports a real version.

`rustfmt.toml`, `rust-toolchain.toml`, `.pre-commit-config.yaml`, `.github/workflows/validate.yml`,
`.editorconfig`, `.markdownlint.yaml` and `.shellcheckrc` are generated from a shared toolchain
outside this repo. An edit to one here is overwritten on the next sync.

## License — GPL-3.0-or-later, inherited from the decoder rather than chosen

Decoding `attributedBody` is the whole read path, and the mature parser for it is
[`imessage-database`](https://crates.io/crates/imessage-database), which is GPL-3.0-or-later.
Linking it makes this a derivative work, so any other license here would be a false claim.

The practical effect is small. The source is already public, and every release points at a tagged
commit, which is what the license asks for. A fork stays GPL-3. msg cannot return to a permissive
license without dropping that dependency.
