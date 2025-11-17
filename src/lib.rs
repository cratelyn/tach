//! a compact cpu monitor.

use {
    self::{
        sentinel::{Recording, Sentinel},
        window::{Height, Width},
    },
    crate::window::Size,
    std::{
        io::{self, Write},
        time::Duration,
    },
};

/// named constants for unicode characters.
mod chars;

/// a meter displaying cpu usage.
mod meter;

/// a stream of statistics measurements.
mod sentinel;

/// abstracts over i/o sources.
mod source;

/// kernel statistics facilities.
///
/// this file provides tools to interact with `/proc/stat`.
pub mod stat;

/// the tui window.
mod window;

/// an instance of the `tach` application.
pub struct App {
    /// the sentinel, observing kernel statistics.
    sentinel: Sentinel,
    /// the number of cpus.
    num_cpus: usize,
    /// the size of the window.
    size: Size,
}

/// A boxed error.
type Error = Box<dyn std::error::Error>;

/// === impl App ===

impl App {
    /// initializes a new application.
    pub fn new() -> Result<Self, Error> {
        let (cols, rows) = crossterm::terminal::size()?;
        let width = Width(cols);
        let height = Height(rows);
        let size = Size { width, height };

        Ok(Self {
            sentinel: Sentinel::new(),
            num_cpus: num_cpus::get(),
            size,
        })
    }

    /// runs the application.
    pub fn run(self) -> Result<(), Error> {
        self.tui().map_err(Into::into)
    }

    /// sleeps until another measurement should be taken.
    fn sleep() {
        const INTERVAL: Duration = Duration::from_secs(1);
        std::thread::sleep(INTERVAL);
    }
}
