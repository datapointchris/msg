mod app;
mod conversation;
mod fixture;
mod thread;
mod ui;

use std::io::{self, IsTerminal};

use anyhow::{Context, bail};
use clap::Parser;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event};
use ratatui::crossterm::execute;

use crate::app::App;

/// Terminal iMessage client.
///
/// Reads chat.db on a Mac over SSH and sends through Shortcuts. Running it with
/// no arguments opens the conversation view.
#[derive(Parser, Debug)]
#[command(name = "msg", version, about, long_about = None)]
struct Cli {}

fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();

    // The view draws to stdout, and stdout is data. Drawing escape codes into
    // a pipe or a file would hand whatever reads it a screenful of noise.
    if !io::stdout().is_terminal() {
        bail!("the conversation view needs a terminal on stdout");
    }

    // Sample conversations: nothing is read from disk and nothing is sent.
    let mut app = App::new(fixture::conversations(), true);

    let mut terminal = ratatui::try_init().context("opening the terminal")?;
    // Without bracketed paste a paste arrives one key at a time, so a pasted
    // tab moves focus and the text after it runs as list keys, `q` included.
    let result = execute!(io::stdout(), EnableBracketedPaste)
        .context("enabling bracketed paste")
        .and_then(|()| {
            disable_paste_on_panic();
            run(&mut terminal, &mut app).context("running the conversation view")
        });
    let _ = execute!(io::stdout(), DisableBracketedPaste);
    ratatui::restore();
    result
}

/// Chains in front of the hook `try_init` installed, which restores the rest of
/// the terminal.
fn disable_paste_on_panic() {
    let restore = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(io::stdout(), DisableBracketedPaste);
        restore(info);
    }));
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.quit {
        terminal.draw(|frame| ui::draw(frame, app))?;
        match event::read()? {
            Event::Key(key) => app.handle_key(key),
            Event::Paste(text) => app.paste(&text),
            _ => {}
        }
    }
    Ok(())
}
