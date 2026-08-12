use std::process::ExitCode;

use clap::Parser;

/// Terminal iMessage client.
///
/// Reads chat.db on a Mac over SSH and sends through Shortcuts. Running it with
/// no arguments opens the conversation view.
#[derive(Parser, Debug)]
#[command(name = "msg", version, about, long_about = None)]
struct Cli {}

fn main() -> ExitCode {
    let _cli = Cli::parse();

    // The TUI is not written. Failing loudly beats opening an empty window and
    // beats exiting 0 with nothing rendered, either of which reads as a working
    // install to whatever shelled out to this. stderr, not stdout: stdout is
    // data, and this is not.
    eprintln!("msg: the conversation view is not implemented yet");
    eprintln!("     tracked as item 409 — read chat.db, send via Shortcuts");
    ExitCode::FAILURE
}
