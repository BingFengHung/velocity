#![allow(dead_code)]

mod app;
mod archive;
mod cli;
mod config;
mod fs;
mod fuzzy;
mod git;
mod graphics;
mod icons;
mod syntax;
mod theme;
mod tui;
mod update;

use app::App;
use clap::Parser;
use cli::Cli;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use fs::open_in_editor;
use std::io::{self, stdout};
use std::panic;
use std::path::PathBuf;
use std::time::Duration;
use tui::{render_ui, TerminalLayout};

fn setup_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let mut out = stdout();
        let _ = out.execute(Show);
        let _ = out.execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
        default_hook(panic_info);
    }));
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    // Handle update commands
    if args.update || args.check_update {
        if let Err(e) = update::check_and_update(args.check_update) {
            eprintln!("❌ 更新失敗: {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    setup_panic_hook();

    let initial_path = args
        .path
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let mut app = App::new(initial_path, args.icons, args.image_protocol, args.all);

    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    let res = run_app(&mut stdout, &mut app);

    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    res
}

fn run_app<W: io::Write>(stdout: &mut W, app: &mut App) -> io::Result<()> {
    let mut needs_render = true;

    loop {
        let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
        let layout = TerminalLayout::calculate(width, height);

        if needs_render {
            app.adjust_scroll(layout.inner_h as usize);
            render_ui(stdout, app, &layout)?;
            needs_render = false;
        }

        if app.should_quit {
            break;
        }

        // If the app wants to open an editor, suspend TUI first
        if let Some(path) = app.pending_open_path.take() {
            // 1. Restore terminal to normal mode
            stdout.execute(Show)?;
            stdout.execute(LeaveAlternateScreen)?;
            disable_raw_mode()?;

            // 2. Run the editor (blocks until it exits)
            let _ = open_in_editor(&path);

            // 3. Re-enter TUI
            enable_raw_mode()?;
            stdout.execute(EnterAlternateScreen)?;
            stdout.execute(Hide)?;

            needs_render = true;
            continue;
        }

        let poll_duration = if app.status_message.is_some() {
            Duration::from_millis(100)
        } else {
            Duration::from_millis(250)
        };

        if event::poll(poll_duration)? {
            match event::read()? {
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Press {
                        app.handle_key(key_event, layout.inner_h as usize);
                        needs_render = true;
                    }
                }
                Event::Resize(_, _) => {
                    needs_render = true;
                }
                _ => {}
            }
        } else if let Some((_, instant)) = app.status_message {
            if instant.elapsed() >= Duration::from_secs(3) {
                app.status_message = None;
                needs_render = true;
            }
        }
    }

    Ok(())
}
