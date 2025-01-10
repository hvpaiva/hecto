//! # Terminal Abstraction Module
//!
//! This module provides a thin layer on top of [crossterm](https://docs.rs/crossterm)
//! for basic terminal operations: enabling/disabling raw mode, clearing the screen,
//! printing text, etc. Instead of writing directly to `stdout()`, it calls
//! [`io_provider::out()`](crate::io_provider::out), which returns a real or fake writer
//! depending on whether we're in test mode. This allows us to capture output in tests
//! without interacting with a real terminal.

use std::io::Write;

use crossterm::{
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

use crate::error::Result;
use crate::io_provider::out;

/// Represents an on‐screen position: (column, row).
///
/// Note that this is *not* the same as a logical location in a text document.
/// The editor or other modules might need to do scrolling or mapping from
/// text lines to terminal rows.
#[derive(Debug, Default, Clone, Copy)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

/// Represents the size of the terminal: (width, height).
#[derive(Debug, Default, Clone, Copy)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

/// Initializes the terminal environment by enabling raw mode, clearing the
/// screen, and moving the cursor to the top‐left.
pub fn initialize() -> Result<()> {
    enable_raw_mode()?;
    clear_screen()?;
    cursor::move_to(Position::default())?;
    execute()
}

/// Disables raw mode and flushes any queued commands before returning.
pub fn terminate() -> Result<()> {
    execute()?;
    disable_raw_mode().map_err(Into::into)
}

/// Returns the current terminal size in (columns, rows) as a [`Size`].
///
/// Internally, crossterm uses `u16`, so we convert them to `usize`.
pub fn size() -> Result<Size> {
    let (width, height) = terminal::size()?;
    Ok(Size {
        width: width.into(),
        height: height.into(),
    })
}

/// Clears the entire terminal screen.
/// (No implicit flush; call [`execute()`] to flush.)
pub fn clear_screen() -> Result<()> {
    crossterm::queue!(out(), Clear(ClearType::All)).map_err(Into::into)
}

/// Clears the current line in the terminal.
/// (No implicit flush; call [`execute()`] to flush.)
pub fn clear_line() -> Result<()> {
    crossterm::queue!(out(), Clear(ClearType::CurrentLine)).map_err(Into::into)
}

/// Prints the given string to the terminal.
/// (No implicit flush; call [`execute()`] to flush.)
pub fn print(s: &str) -> Result<()> {
    crossterm::queue!(out(), Print(s)).map_err(Into::into)
}

/// Flushes (executes) any queued terminal commands.
///
/// In normal usage, you might call this infrequently. For instance, you might
/// enqueue several prints or clears, then flush once.
pub fn execute() -> Result<()> {
    out().flush().map_err(Into::into)
}

pub mod cursor {
    //! # `cursor` Submodule
    //!
    //! Provides methods to manipulate the cursor using crossterm commands, but
    //! writes to [`io_provider::out()`](crate::io_provider::out) so that in tests
    //! we can capture the output rather than altering the real terminal.

    use super::{Position, Result};
    use crate::io_provider::out;
    use crossterm::cursor::{Hide, MoveTo, Show};

    /// Hides the terminal cursor (does not flush automatically).
    /// (No implicit flush; call [`terminal::execute()`] to flush.)
    pub fn hide() -> Result<()> {
        crossterm::queue!(out(), Hide).map_err(Into::into)
    }

    /// Shows the terminal cursor (does not flush automatically).
    /// (No implicit flush; call [`terminal::execute()`] to flush.)
    pub fn show() -> Result<()> {
        crossterm::queue!(out(), Show).map_err(Into::into)
    }

    /// Moves the cursor to the given [`Position`]: (col, row).
    ///
    /// If `col` or `row` exceed `u16::MAX`, it returns a conversion error.
    /// (No implicit flush; call [`terminal::execute()`] to flush.)
    pub fn move_to(pos: Position) -> Result<()> {
        let col_u16: u16 = pos.col.try_into()?;
        let row_u16: u16 = pos.row.try_into()?;
        crossterm::queue!(out(), MoveTo(col_u16, row_u16)).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    //! # Terminal Unit Tests
    //!
    //! These tests confirm that the terminal module writes the expected ANSI
    //! sequences to our fake writer (in test mode). We then parse those output
    //! bytes to verify correctness. It also checks for certain functions that
    //! do not necessarily write sequences (like `size()`).

    use super::*;
    use crate::io_provider::take_out_contents;

    /// Checks if `initialize()` produces sequences for raw mode enable, screen clear,
    /// and moving cursor to (0, 0). We primarily verify the screen clear and cursor move
    /// since raw mode enabling does not produce a visible ANSI code in crossterm.
    #[test]
    fn test_initialize() {
        initialize().unwrap();

        let contents = take_out_contents();
        let output = String::from_utf8_lossy(&contents);

        // Typically, crossterm clearing the screen might include "[2J"
        // and moving the cursor to the top-left is often something like "[H" or "[1;1H".
        assert!(
            output.contains("[2J"),
            "Expected '[2J' for clearing screen in initialize()"
        );
        assert!(
            output.contains("[;H") || output.contains("[1;1H"),
            "Expected move to top-left (often '[H' or '[1;1H'); got: {output}"
        );
    }

    /// Tests that `terminate()` doesn't produce an error. It should flush queued commands
    /// and disable raw mode (which doesn't typically generate visible ANSI codes).
    #[test]
    fn test_terminate() {
        // We won't queue anything special here; just ensure no error
        terminate().unwrap();

        let contents = take_out_contents();
        let output = String::from_utf8_lossy(&contents);

        // Disabling raw mode does not typically produce ANSI sequences that remain in the buffer.
        // So we just confirm there's nothing suspicious or no error occurred.
        assert!(
            !output.contains("[2J"),
            "Did not expect a second screen clear in terminate()"
        );
    }

    /// Checks the `size()` function. This calls `crossterm::terminal::size()`,
    /// which depends on the environment. We primarily verify it doesn't error
    /// and that the reported size is not zero.
    #[test]
    fn test_size() {
        let sz = size().unwrap();
        // In many terminals, width and height should be > 0.
        // But there's no universal guarantee in all CI or container environments.
        // We do a basic sanity check:
        assert!(
            sz.width > 0,
            "Expected terminal width to be > 0 (actual: {})",
            sz.width
        );
        assert!(
            sz.height > 0,
            "Expected terminal height to be > 0 (actual: {})",
            sz.height
        );
    }

    #[test]
    fn test_clear_screen() {
        clear_screen().unwrap();
        execute().unwrap();

        let contents = take_out_contents();
        let output = String::from_utf8_lossy(&contents);

        // crossterm typically uses "\x1B[2J" to clear the screen
        assert!(
            output.contains("[2J"),
            "Expected [2J in clear screen command"
        );
    }

    /// Tests clearing the current line; crossterm emits "[2K".
    #[test]
    fn test_clear_line() {
        clear_line().unwrap();
        execute().unwrap();

        let contents = take_out_contents();
        let output = String::from_utf8_lossy(&contents);
        dbg!(&output);

        // Usually, "[K" is the ANSI for "Clear line"
        // But crossterm often uses "[2K" for "Clear current line"
        assert!(output.contains("[2K"), "Expected [2K in clear line command");
    }

    #[test]
    fn test_print() {
        print("Hello, world!").unwrap();
        execute().unwrap();

        let contents = take_out_contents();
        let output = String::from_utf8_lossy(&contents);
        assert!(output.contains("Hello, world!"));
    }

    #[test]
    fn test_cursor_hide_show() {
        cursor::hide().unwrap();
        cursor::show().unwrap();
        execute().unwrap();

        let contents = take_out_contents();
        let output = String::from_utf8_lossy(&contents);
        // crossterm uses "[?25l" for Hide, "[?25h" for Show
        assert!(output.contains("[?25l"), "Expected hide command [\"?25l\"]");
        assert!(output.contains("[?25h"), "Expected show command [\"?25h\"]");
    }

    #[test]
    fn test_cursor_move_to_ok() {
        cursor::move_to(Position { col: 10, row: 5 }).unwrap();
        execute().unwrap();

        let contents = take_out_contents();
        let output = String::from_utf8_lossy(&contents);
        // Crossterm with MoveTo(10,5) often => "\x1B[6;11H" (ESC [ row+1 ; col+1 H)
        assert!(
            output.contains("[6;11H"),
            "Expected some form of row=5+1; col=10+1"
        );
    }

    #[test]
    fn test_cursor_move_to_overflow() {
        // col=70000 => exceeds u16::MAX => should fail
        let err = cursor::move_to(Position { col: 70000, row: 5 })
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("out of range integral type conversion attempted")
                || err.contains("TryFromIntError"),
            "Expected a TryFromIntError or equivalent for overflow; got: {err}"
        );
    }
}
