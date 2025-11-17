//! a compact cpu monitor.

use {
    self::sentinel::{Recording, Sentinel},
    std::{
        io::{self, Write},
        time::Duration,
    },
};

/// a meter displaying cpu usage.
mod meter;

/// a stream of statistics measurements.
mod sentinel;

/// abstracts over i/o sources.
mod source;

/// kernel statistics facilities.
///
/// this file provides tools to interact with `/proc/stat`.
mod stat;

/// the tui window.
mod window;

/// an instance of the `tach` application.
pub struct App {
    /// the sentinel, observing kernel statistics.
    sentinel: Sentinel,
}

/// A boxed error.
type Error = Box<dyn std::error::Error>;

/// === impl App ===

impl App {
    /// initializes a new application.
    pub fn new() -> Self {
        Self {
            sentinel: Sentinel::new(),
        }
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
