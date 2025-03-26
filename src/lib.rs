//! a compact cpu monitor.

use {
    self::{
        meter::Meter,
        sentinel::{Recording, Sentinel},
        stat::StatReadError,
    },
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

pub struct App {
    sentinel: Sentinel,
}

/// === impl App ===

impl App {
    /// initializes a new application.
    pub fn new() -> Self {
        Self {
            sentinel: Sentinel::new(),
        }
    }

    /// runs the application.
    pub fn run(self) -> Result<(), StatReadError> {
        let Self { mut sentinel } = self;

        loop {
            sentinel.observe()?.map(Self::draw);
            Self::sleep();
        }
    }

    fn draw(
        Recording {
            start: _,
            end: _,
            system,
            cpus,
        }: Recording,
    ) -> io::Result<()> {
        let percentage = system.percentage();

        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();

        Meter {
            name: "system".to_owned(),
            value: percentage as usize,
            width: 100,
        }
        .draw(&mut stdout)?;
        write!(&mut stdout, "\n")?;

        for (id, measurement) in cpus {
            let percentage = measurement.percentage();
            Meter {
                name: format!("{id:?}"),
                value: percentage as usize,
                width: 100,
            }
            .draw(&mut stdout)?;
            write!(&mut stdout, "\n")?;
        }

        Ok(())
    }

    fn sleep() {
        const INTERVAL: Duration = Duration::from_secs(1);
        std::thread::sleep(INTERVAL);
    }
}
