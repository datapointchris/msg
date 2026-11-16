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

const EXAMPLES: &str = "\
Examples:
  msg             open the view, with the list titled \"Conversations (sample data)\"
  msg --version   the release's tag; a build from source prints 0.0.0

In the view, ? on the list shows every key, and q quits.";

#[derive(Parser, Debug)]
#[command(name = "msg", version, about, after_help = EXAMPLES)]
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

#[cfg(test)]
mod tests {
    use clap::{Command, CommandFactory};

    use super::Cli;

    /// Every command a help screen lists and a reader can run, whatever its depth.
    fn listed_leaves(command: &Command) -> Vec<&Command> {
        if command.is_hide_set() {
            return Vec::new();
        }
        let children: Vec<&Command> = command.get_subcommands().filter(|c| c.get_name() != "help").collect();
        if children.is_empty() {
            return vec![command];
        }
        children.into_iter().flat_map(listed_leaves).collect()
    }

    /// A reader who reaches a command by tab-completion lands on its help, so
    /// each one carries a command to copy. Reads the field, not rendered help.
    #[test]
    fn every_listed_leaf_carries_an_example() {
        let root = Cli::command();
        let leaves = listed_leaves(&root);
        assert!(!leaves.is_empty());
        let missing: Vec<&str> = leaves
            .iter()
            .filter(|c| !c.get_after_help().is_some_and(|h| h.to_string().contains("Examples:")))
            .map(|c| c.get_name())
            .collect();
        assert!(missing.is_empty(), "commands with no Examples block: {missing:?}");
    }
}
