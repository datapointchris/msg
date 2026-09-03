# CLAUDE.md

Guidance for Claude Code working in this repository.

## What this is

A terminal iMessage client in Rust. It follows the author's Rust conventions, which are kept
outside this repo and are where the formatting width, the clippy posture, the error-handling split,
the module layout and the release shape are decided. Nothing in this file restates them.

The repo is scaffolded. `src/main.rs` is a clap parser and nothing else: `--version` and `--help`
work, and a bare invocation reports the conversation view as unimplemented on stderr and exits 1.
Only what the shape forces is there — no verb grammar, because that is a design decision rather
than scaffolding.

**There are no tests, deliberately.** There is no logic to test, and a test written against a
placeholder only to turn a line green is worse than no test. The first real test arrives with the
first real behavior, and the one worth writing early pins the contract that stdout carries data
and nothing else.

## Read this before choosing where to put the read path

**The architecture is not settled, and picking wrong is a rewrite rather than a refactor.**
`imessage-database` is a dependency, but only part of it is reachable depending on which way this
goes:

| | Needs | Gives |
| --- | --- | --- |
| `tables::table::get_connection(path: &Path)` | a **local** `chat.db` file | the rich layer — messages, attachments, reactions, threads, edited messages |
| `util::streamtyped::parse(Vec<u8>)` | a raw blob, no connection | the `attributedBody` decoder alone |

The verified read channel is `sqlite3 -readonly` **over SSH** against the Mac, which is live and
returns messages seconds old. That is remote, so it cannot feed `get_connection`. Either the
database is copied locally and the rich layer is used against a snapshot, or the query stays remote
and only the blob decoder is used — in which case the schema work is yours.

Decide it before writing the module, not during. It is tracked as its own item.

## The two things most likely to waste your time

**Do not reach for AppleScript.** It is broken on macOS 15.7.7 in both directions and was verified
that way across a clean Messages restart, in all three sending forms. `send` hangs with rc=124 and
transmits nothing; `count of chats` and `count of services` hang; `buddies` is gone from the
dictionary. The working channels are `sqlite3 -readonly` for reads and `shortcuts run` for sends,
both over plain SSH.

**Do not scrape `hex(attributedBody)` with a regex.** It is the widely-posted answer and it is
regex over structured data, which breaks on attachments, tapbacks and link previews. 92% of
messages have no usable `text` column, so this path is not an edge case — it is the main one. How
to decode it properly is an open decision recorded in the project tracker, not something to settle
inline.

## Shortcuts gotchas, each paid for once

- `shortcuts sign` discriminates on file **extension**, not content. The same binary plist fails as
  `.plist` and succeeds as `.shortcut` or `.wflow`.
- `Recipients` must be a serialized `WFContactFieldValue` vCard. A bare phone-number string array
  still delivers, but pops a recipient-picker dialog.
- `ShowWhenRun` must be `0`, or every send opens the compose sheet.
- Importing a shortcut always needs a human click. That is a deliberate Apple gate against a remote
  process installing a send-capable automation, and there is no way around it.
- `shortcuts run` returns the shortcut's own output on stdout.

## Generated files are not yours to edit

`rustfmt.toml`, `.pre-commit-config.yaml`, `.github/workflows/validate.yml`, `.editorconfig`,
`.markdownlint.json`, `.markdownlintignore` and `.shellcheckrc` are generated from a shared
toolchain outside this repo. Change them at that source and re-sync; editing them here is undone on
the next sync.

`Cargo.toml`'s `version = "0.0.0"` is deliberate. The tag is the version and CI sets it at build
time, so do not bump it.

## Two more that read as mistakes and are not

**The license is GPL-3.0-or-later and was inherited, not chosen.** `imessage-database` is
GPL-3.0-or-later and this links it, so any other license would be a false claim. It is reversible
only by dropping that dependency. Nothing catches GPL from a binary, so the direction to watch is
outward: anything lifted *out* of this repo into a shared library carries GPL-3 into everything
linking that library.

**92% of messages have no usable `text` column.** Measured: 759 of 9814 rows. Code that treats
`text` as the primary source and `attributedBody` as the fallback has it exactly backwards — the
decode path is the normal path.
