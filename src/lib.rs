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
        const BORDER: char = '│';

        let meter_width = {
            let columns = 150; // XXX(hardcoded for now)
            let ncpus = cpus.len() - 1;
            let delims = ncpus - 1;
            let per_cpu = (columns - delims) / ncpus;
            per_cpu
        };

        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();

        write!(&mut stdout, "  ")?;

        write!(&mut stdout, "\x1B[90m")?;
        write!(&mut stdout, "{BORDER}")?;
        write!(&mut stdout, "\x1B[0m")?;

        for (_, measurement) in cpus {
            // set the foreground color via `ESC[{..}m`.
            // red: 31
            // yellow: 33
            // green: 32
            // grey: 90
            write!(&mut stdout, "\x1B[31m")?;

            Meter {
                value: measurement.active() / measurement.total(),
                width: meter_width,
            }
            .draw(&mut stdout)?;

            // reset the foreground color via `ESC[{..}m`.
            write!(&mut stdout, "\x1B[0m")?;

            write!(&mut stdout, "\x1B[90m")?;
            write!(&mut stdout, "{BORDER}")?;
            write!(&mut stdout, "\x1B[0m")?;
        }
        write!(&mut stdout, " ")?;

        write!(&mut stdout, "\x1B[32m")?;
        let percentage = system.percentage();
        write!(&mut stdout, " {percentage:02}%")?;
        write!(&mut stdout, "\x1B[0m")?;

        write!(&mut stdout, "\n")?;

        Ok(())
    }

    fn sleep() {
        const INTERVAL: Duration = Duration::from_secs(1);
        std::thread::sleep(INTERVAL);
    }
}
