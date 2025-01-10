use crate::{
    buffer::Buffer,
    error::Result,
    terminal::{self, Size},
};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Default, Clone)]
pub struct View {
    pub buffer: Buffer,
}

impl View {
    pub fn load(&mut self, file_name: &str) {
        if let Ok(buffer) = Buffer::load(file_name) {
            self.buffer = buffer;
        }
    }

    /// Render all the rows of the editor’s screen content.
    ///
    /// Clears each line, then either render a welcome message row or an empty row
    /// (with a “~” in the first column).
    pub fn render(&self) -> Result<()> {
        if self.buffer.is_empty() {
            Self::render_welcome()?;
        } else {
            self.render_buffer()?;
        }
        Ok(())
    }

    fn render_welcome() -> Result<()> {
        let Size { height, .. } = terminal::size()?;

        for row in 0..height {
            terminal::clear_line()?;

            if row == height.saturating_div(3) {
                render_welcome_row()?;
            } else {
                render_empty_row()?;
            }

            if row.saturating_add(1) < height {
                terminal::print("\r\n")?;
            }
        }
        Ok(())
    }

    fn render_buffer(&self) -> Result<()> {
        let Size { height, .. } = terminal::size()?;

        for row in 0..height {
            terminal::clear_line()?;

            if let Some(line) = self.buffer.get(row) {
                terminal::print(line)?;
            } else {
                render_empty_row()?;
            }

            if row.saturating_add(1) < height {
                terminal::print("\r\n")?;
            }
        }
        Ok(())
    }
}

/// Render an empty row, indicated by a single “~” in the leftmost column.
fn render_empty_row() -> Result<()> {
    terminal::print("~")
}

/// Render the “welcome message” row, centered horizontally.
/// (We don’t require perfect centering; it’s just approximate.)
fn render_welcome_row() -> Result<()> {
    let mut welcome_message = format!("{NAME} editor -- version {VERSION}");
    let width = terminal::size()?.width;
    let len = welcome_message.len();

    let padding = (width.saturating_sub(len)).saturating_div(2);
    // We put a “~” at the start, then some spaces, then the message.
    let leading_spaces = " ".repeat(padding.saturating_sub(1));
    welcome_message = format!("~{leading_spaces}{welcome_message}");

    // If the message is bigger than the width, we truncate
    welcome_message.truncate(width);
    terminal::print(&welcome_message)
}

#[cfg(test)]
mod tests {
    use crate::{io_provider::take_out_contents, terminal};

    #[test]
    fn test_render_welcome() {
        // We'll call `super::render_welcome()` directly and check
        // the buffer for something like "~    <PackageName> editor -- version <Version>".
        super::render_welcome_row().unwrap();
        terminal::execute().unwrap();

        let contents = take_out_contents();
        let out = String::from_utf8_lossy(&contents);
        // We expect something like "~ <some spaces>my_crate editor -- version 1.0.0"
        assert!(
            out.contains("editor -- version"),
            "Expected welcome message in output"
        );
    }

    #[test]
    fn test_render_empty() {
        super::render_empty_row().unwrap();
        terminal::execute().unwrap();

        let contents = take_out_contents();
        let out = String::from_utf8_lossy(&contents);
        // Should just print "~"
        assert!(out.contains('~'), "Expected a single '~' for empty row");
    }

    #[test]
    fn test_render() {
        let view = super::View::default();

        view.render().unwrap();
        terminal::execute().unwrap();

        let contents = take_out_contents();
        let out = String::from_utf8_lossy(&contents);
        // We'll check at least for a bunch of "~" characters,
        // as many as the terminal height (but we can't be certain what the terminal size is).
        assert!(
            out.contains('~'),
            "Expected at least some empty row symbols (~)"
        );
        assert!(
            out.contains("editor -- version"),
            "Expected the welcome row somewhere in the output"
        );
    }
}
