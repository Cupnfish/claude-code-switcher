//! Apply-focused TUI: a keyboard-driven screen for the "second half" of an
//! apply — pick a key, tune options, preview, apply. Gathers the user's intent
//! and returns an [`ApplySelection`]; it does not touch settings.json itself.

mod app;
mod input;
mod view;

pub use app::{ApplySelection, Outcome};

use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::prefs::Prefs;
use crate::templates::TemplateType;

use app::App;

/// Open the apply TUI for a resolved target.
///
/// Returns `Ok(Some(selection))` on apply, `Ok(None)` if the user cancelled.
pub fn run_apply_tui(
    template_type: TemplateType,
    target: String,
    display_name: String,
    current_label: String,
    prefs: &Prefs,
) -> Result<Option<ApplySelection>> {
    // Build state first (may error before we touch the terminal).
    let mut app = App::new(template_type, target, display_name, current_label, prefs)?;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = (|| -> Result<Option<ApplySelection>> {
        loop {
            terminal.draw(|f| view::render(f, &app))?;
            let ev = match event::read() {
                Ok(e) => e,
                Err(_) => break Ok(None),
            };
            if let Event::Key(k) = ev {
                if k.kind == KeyEventKind::Release {
                    continue;
                }
                match app.handle_event(k) {
                    Outcome::Continue => continue,
                    Outcome::Apply(sel) => break Ok(Some(sel)),
                    Outcome::Quit => break Ok(None),
                }
            }
        }
    })();

    // Restore the terminal regardless of outcome.
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen, Show).ok();
    terminal.show_cursor().ok();

    result
}
