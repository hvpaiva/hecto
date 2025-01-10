//! # Core Editor Module
//!
//! This module manages the core event loop, user input, and high‐level editing
//! logic. It also defines a [`Location`] type, which represents a place in the
//! **document** (not necessarily on‐screen).
//!
//! ## Responsibilities
//! - **Run** the main loop that reads keyboard events and updates editor state.
//! - **Track** whether the user wants to quit (`should_quit`).
//! - **Maintain** the current [Location] in the document (i.e., line and column
//!   in the text).
//! - **Delegate** terminal interaction (drawing, cursor movements) to the
//!   [`terminal`](crate::terminal) module.
//! - **Handle** special keys (e.g., arrow keys, page up/down) to move the
//!   [Location] around.

use std::{cmp::min, env};

use crate::{
    error::Result,
    terminal::{self, cursor, Position, Size},
    viewer::View,
};

use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Represents a specific place in the document (line/column in text).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Location {
    col: usize,
    row: usize,
}

impl From<Location> for Position {
    fn from(location: Location) -> Self {
        Position {
            col: location.col,
            row: location.row,
        }
    }
}

/// The main editor state and event loop controller.
///
/// Stores whether we should quit and the current [`Location`] in the text.
/// Exposes a [`run()`][Editor::run] method to start the REPL.
#[derive(Debug, Default, Clone)]
pub struct Editor {
    /// If set to `true`, the editor will exit on the next refresh.
    should_quit: bool,
    /// The current logical “Location” in the text (not necessarily on‐screen).
    location: Location,

    view: View,
}

impl Editor {
    /// Runs the main read‐evaluate‐print loop (REPL).
    ///
    /// Continuously:
    /// 1. Refreshes/redraws the terminal.
    /// 2. Reads user input events.
    /// 3. Updates state or decides to quit.
    ///
    /// When `should_quit` is set to `true`, the loop breaks and we terminate.
    pub fn run(&mut self) -> Result<()> {
        terminal::initialize()?;
        self.handle_args();
        self.repl()?;
        terminal::terminate()
    }

    fn handle_args(&mut self) {
        let args: Vec<String> = env::args().collect();
        if let Some(filename) = args.get(1) {
            self.view.load(filename);
        }
    }

    /// Internal REPL loop.
    /// Exits if `should_quit` becomes `true`.
    fn repl(&mut self) -> Result<()> {
        loop {
            self.refresh()?;
            if self.should_quit {
                break;
            }

            let event = read()?;
            self.handle_event(&event)?;
        }
        Ok(())
    }

    /// Interprets a single [`Event`], updating the editor’s state accordingly.
    ///
    /// For example, pressing `Ctrl+Q` sets `should_quit = true`.
    /// Arrow keys and other navigation keys are passed to [`move_cursor`].
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            match code {
                KeyCode::Char('q') => {
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        self.should_quit = true;
                    }
                }
                KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::Home
                | KeyCode::End
                | KeyCode::PageUp
                | KeyCode::PageDown => {
                    self.move_cursor(*code)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Redraws the screen, optionally clearing it and printing “Goodbye.” if
    /// `should_quit` is true, or drawing the editor rows otherwise.
    ///
    /// Finally, restores the cursor position and flushes output.
    fn refresh(&self) -> Result<()> {
        cursor::hide()?;
        // Move cursor to top-left before drawing
        cursor::move_to(Position::default())?;
        if self.should_quit {
            terminal::clear_screen()?;
            terminal::print("Goodbye.\r\n")?;
        } else {
            self.view.render()?;
            // Move cursor to the editor’s current logical location
            cursor::move_to(self.location.into())?;
        }
        cursor::show()?;
        terminal::execute()
    }

    /// Moves the editor’s logical location (row/col) in response to arrow keys, etc.
    ///
    /// The boundaries are clamped by the current `terminal::size()`. If the user tries
    /// to move beyond the screen width/height, we saturate to the edge.
    fn move_cursor(&mut self, key: KeyCode) -> Result<()> {
        let Location { mut col, mut row } = self.location;
        let Size { height, width } = terminal::size()?;

        match key {
            KeyCode::Up => {
                row = row.saturating_sub(1);
            }
            KeyCode::Down => {
                row = min(height.saturating_sub(1), row.saturating_add(1));
            }
            KeyCode::Left => {
                col = col.saturating_sub(1);
            }
            KeyCode::Right => {
                col = min(width.saturating_sub(1), col.saturating_add(1));
            }
            KeyCode::PageUp => {
                row = 0;
            }
            KeyCode::PageDown => {
                row = height.saturating_sub(1);
            }
            KeyCode::Home => {
                col = 0;
            }
            KeyCode::End => {
                col = width.saturating_sub(1);
            }
            _ => (),
        }

        self.location = Location { col, row };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! # Editor Unit Tests
    //!
    //! Here we validate the behavior of our `Editor` struct, including:
    //! - Key event handling (`handle_event`)
    //! - The drawing of rows (welcome message, empty rows)
    //! - The `refresh` method (which hides the cursor, draws, etc.)
    //!
    //! In a real scenario, we might also want to mock the `crossterm::event::read()`
    //! calls for testing `repl()`. However, for this simple example, we focus on the
    //! logic and rendering aspects, capturing any terminal output with
    //! our `io_provider::out()` approach.

    use super::*;
    use crate::io_provider::take_out_contents;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_handle_event_quit() {
        // Pressing Ctrl+Q sets `should_quit = true`
        let mut editor = Editor::default();
        let evt = crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });
        editor.handle_event(&evt).unwrap();
        // Check if we set `should_quit`
        assert!(
            editor.should_quit,
            "Expected `should_quit` to be true after Ctrl+Q"
        );
    }

    #[test]
    fn test_handle_event_arrow_keys() {
        // Up arrow => decrement row
        let mut editor = Editor::default();

        let evt_down = crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });
        editor.handle_event(&evt_down).unwrap();
        assert_eq!(
            editor.location,
            Location { col: 0, row: 1 },
            "Expected row to decrement on Up"
        );
    }

    #[test]
    fn test_move_cursor_bounds() {
        // We can call `move_cursor` directly to test boundary conditions.
        let mut editor = Editor::default();

        // KeyCode::Up should saturate at 0 => no negative row
        editor.move_cursor(KeyCode::Up).unwrap();
        assert_eq!(editor.location.row, 0, "Row should remain 0 on Up at top");

        // Same for KeyCode::Left
        editor.move_cursor(KeyCode::Left).unwrap();
        assert_eq!(
            editor.location.col, 0,
            "Col should remain 0 on Left at leftmost"
        );

        // We'll also try something that attempts to go beyond the max row/col.
        // We can't easily know the terminal size in a test, but let's assume it's
        // at least 5x5. We'll forcibly set the editor's location near the edge
        // and call KeyCode::Right, KeyCode::Down a few times.

        editor.location = Location {
            col: 10000,
            row: 10000,
        };
        editor.move_cursor(KeyCode::Right).unwrap();
        // We can't assert exact max, but we know `col` won't exceed `width-1`.
        // This is more an integration test scenario, but let's do a minimal check:
        assert!(
            editor.location.col <= 10000,
            "Cursor should saturate horizontally"
        );
    }

    #[test]
    fn test_refresh_goodbye() {
        // If `should_quit` is true, refresh() clears screen and prints "Goodbye."
        let editor = Editor {
            should_quit: true,
            ..Editor::default()
        };

        editor.refresh().unwrap();

        let contents = take_out_contents();
        let out = String::from_utf8_lossy(&contents);
        assert!(
            out.contains("Goodbye."),
            "Expected 'Goodbye.' if should_quit=true"
        );
    }

    #[test]
    fn test_refresh_normal() {
        // If `should_quit` is false, refresh draws rows, then repositions cursor.
        let editor = Editor::default();
        editor.refresh().unwrap();

        let contents = take_out_contents();
        let out = String::from_utf8_lossy(&contents);
        assert!(
            out.contains("editor -- version") || out.contains('~'),
            "Expected some row drawing output if not quitting"
        );
    }
}
