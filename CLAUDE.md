# CLAUDE.md

Guidance for Claude Code working in this repository.

## What this is

A terminal iMessage client in Rust, and the first repo built to `standards/rust.md`. Read that file
before writing code here — it is where the formatting width, the clippy posture, the error-handling
split, the module layout and the release shape are decided, with the reasoning. Nothing in this
file restates it.

The repo is scaffolded. `src/main.rs` is a clap parser and nothing else: `--version` and `--help`
work, and a bare invocation reports the conversation view as unimplemented on stderr and exits 1.
Only what the shape forces is there — no verb grammar, because that is a design decision rather
than scaffolding.

**There are no tests, deliberately.** There is no logic to test, and a test written against a
placeholder to turn a line green is the thing `standards/testing.md` exists to stop. The first real
test arrives with the first real behaviour, and the one worth writing early
pins the machine contract: stdout carries data and nothing else.

## The two things most likely to waste your time

**Do not reach for AppleScript.** It is broken on macOS 15.7.7 in both directions and was verified
that way across a clean Messages restart, in all three sending forms. `send` hangs with rc=124 and
transmits nothing; `count of chats` and `count of services` hang; `buddies` is gone from the
dictionary. The working channels are `sqlite3 -readonly` for reads and `shortcuts run` for sends,
both over plain SSH.

**Do not scrape `hex(attributedBody)` with a regex.** It is the widely-posted answer and it is
regex over structured data, which breaks on attachments, tapbacks and link previews. 92% of
messages have no usable `text` column, so this path is not an edge case — it is the main one. How
to decode it properly is an open decision recorded on the `icb` item, not something to settle
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
`.markdownlint.json`, `.markdownlintignore` and `.shellcheckrc` all come from `forge`. Change them
in `~/tools/forge/pre-commit/` and re-run the die; editing them here is undone on the next sync.

`Cargo.toml`'s `version = "0.0.0"` is deliberate. Do not bump it.

## Where the work is tracked

The `imessage gateway and TUI` project in `icb`. That project also carries the Rust-as-a-fleet-stack
items, so its scope is wider than its name.
