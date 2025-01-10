#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]

use editor::Editor;
use error::Result;

mod editor;
mod error;
pub mod io_provider;
mod terminal;

fn main() -> Result<()> {
    Editor::default().run()?;
    Ok(())
}
