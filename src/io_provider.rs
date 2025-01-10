//! # `io_provider.rs`
//!
//! This module provides an `out()` function that returns a writer. In production
//! (non‐test mode), it returns the real `stdout()`. In test mode, it returns a
//! fake writer that stores all output in memory. This allows us to capture and
//! inspect the output for unit tests without printing to the real terminal.

#[cfg(not(test))]
use std::io::Stdout;

/// Returns `stdout` in non‐test mode.
#[cfg(not(test))]
#[must_use]
pub fn out() -> Stdout {
    std::io::stdout()
}

#[cfg(test)]
use std::cell::RefCell;

#[cfg(test)]
use std::io::Write;

// A thread‐local buffer that stores all output (in test mode).
//
// Using a thread‐local ensures that parallel tests do not clash with each other.
#[cfg(test)]
thread_local! {
    static FAKE_OUT: RefCell<Vec<u8>> = const { RefCell::new(vec![]) };
}

/// Returns a `FakeOut` writer in test mode, which writes to `FAKE_OUT`.
#[cfg(test)]
#[must_use]
pub fn out() -> FakeOut {
    FakeOut
}

/// A fake writer that appends data to the thread‐local `FAKE_OUT` buffer.
#[cfg(test)]
pub struct FakeOut;

#[cfg(test)]
impl Write for FakeOut {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        FAKE_OUT.with(|b| {
            b.borrow_mut().extend_from_slice(buf);
        });
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Flushing does nothing special here
        Ok(())
    }
}

/// Takes (removes) all data from the thread‐local buffer and returns it.
/// This is useful after calling terminal functions in tests, so we can see
/// exactly what was written to the screen (in memory).
#[cfg(test)]
#[must_use]
pub fn take_out_contents() -> Vec<u8> {
    FAKE_OUT.with(|b| b.replace(vec![]))
}
