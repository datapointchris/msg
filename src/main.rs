mod app;
mod fixture;
mod thread;
mod ui;

use std::io::{self, IsTerminal};

use anyhow::{Context, bail};
use clap::Parser;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event};

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

    // The view shows built-in sample conversations until the chat.db read path
    // exists. Nothing is read from disk and nothing is sent.
    let mut app = App::new(fixture::conversations());

    let mut terminal = ratatui::try_init().context("opening the terminal")?;
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result.context("running the conversation view")
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.quit {
        terminal.draw(|frame| ui::draw(frame, app))?;
        if let Event::Key(key) = event::read()? {
            app.handle_key(key);
        }
    }
    Ok(())
}
