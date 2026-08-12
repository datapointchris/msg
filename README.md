# msg

Terminal iMessage client. Reads `chat.db` on a Mac over SSH, sends through Shortcuts, and renders
the result as a TUI laid out like Messages — conversation list on the left, thread on the right,
type to send. Runs from a Linux terminal against a Mac that stays on.

**Scaffolded, not built.** There is a binary and it parses arguments, but the conversation view is
not written — running `msg` reports that and exits non-zero rather than opening an empty window.
What it is *for* is settled and measured; what it *does* is not written yet.

## Why it exists twice over

It is a tool worth having, and it is the author's first Rust project built to a full set of
conventions rather than as a one-off — release shape, lint posture and toolchain pinning included.

## How it will work

Two channels to the Mac, both verified working on macOS 15.7.7 before any code was written:

| Direction | Mechanism |
| --- | --- |
| Read | `sqlite3 -readonly ~/Library/Messages/chat.db` over plain SSH |
| Send | `shortcuts run <name> -i <file>` over plain SSH |

Reading works because sshd already holds Full Disk Access. Sending works unattended with no GUI
session and no sudo.

**AppleScript is not an option and should not be retried.** On 15.7.7 `send` compiles against the
live dictionary, hangs, and transmits nothing. Enumeration is broken the same way — `count of
chats` hangs and `buddies` has left the dictionary — so conversation discovery has to come from
SQLite regardless.

**92% of messages have no readable `text` column.** Of 9814 messages on the reference machine, 759
carry usable text; the rest exist only as `attributedBody`, a serialized NSAttributedString
typedstream. It is not a plist and it is not reachable from SQL, so decoding it is the core of the
project rather than a detail of it. Which decoder — a GPL-3 crate, an MIT one at 0.1.0, or a
hand-written parser against the published format — is the open decision.

## Toolchain

Everything here is generated or standard. `rustfmt.toml`, `.pre-commit-config.yaml`,
`.github/workflows/validate.yml`, `.editorconfig` and the rest are generated from a shared
toolchain and are not hand-edited — change them at that source and re-sync.

```bash
cargo build            # rust-toolchain.toml pins 1.97.1; rustup fetches it on first use
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt
cargo test
```

`Cargo.toml` carries `version = "0.0.0"` deliberately and it is never bumped in git. The tag is the
version, and CI sets it in the tag-checked-out tree before building.

## Installing

Not released yet. When it is, it ships as a GitHub release with `cargo-binstall` metadata:

```bash
cargo binstall msg
```

It does not publish to crates.io — the name is taken there by an unrelated XSI message-queue
binding, and a personal tool gains nothing from the registry.
