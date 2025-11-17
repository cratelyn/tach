use {
    super::*,
    crossterm::{
        ExecutableCommand, QueueableCommand, cursor,
        style::{self, Stylize},
        terminal,
    },
    std::collections::VecDeque,
};

impl App {
    pub fn tui(self) -> Result<(), crate::Error> {
        let Self { mut sentinel } = self;

        Self::clear()?;

        let (cols, rows) = crossterm::terminal::size()?;

        let mut recordings = VecDeque::new();
        loop {
            Self::border(cols, rows)?;

            if let Some(Recording {
                start: _,
                end: _,
                system: _,
                cpus,
            }) = sentinel.observe()?
            {
                for (cpu, _) in cpus.iter() {
                    io::stdout()
                        .queue(cursor::MoveTo(((cpu.as_u16() * 10) + 2) as u16, 2))?
                        .queue(style::PrintStyledContent(
                            format!("cpu{}", cpu.as_u16()).grey(),
                        ))?;
                }

                recordings.push_back(cpus);
                if recordings.len() > (rows - 6) as usize {
                    recordings.pop_front();
                }

                for (row, r) in recordings.iter().enumerate() {
                    for (cpu, measurement) in r.iter() {
                        io::stdout()
                            .queue(cursor::MoveTo(
                                ((cpu.as_u16() * 10) + 2) as u16,
                                (row + 4) as u16,
                            ))?
                            .queue(style::PrintStyledContent(
                                format!("{}", measurement.percentage(),).green(),
                            ))?;
                    }
                }
            }

            io::stdout().queue(cursor::Hide)?;

            io::stdout().flush()?;
            Self::sleep();
        }
    }

    /// clears the screen.
    fn clear() -> Result<(), io::Error> {
        io::stdout()
            .execute(terminal::Clear(terminal::ClearType::All))
            .map(drop)
    }

    // XXX(kate): this could be prettier.
    fn border(cols: u16, rows: u16) -> Result<(), io::Error> {
        let mut stdout = io::stdout();
        for y in 0..rows {
            for x in 0..cols {
                if (y == 0 || y == rows - 1) || (x == 0 || x == cols - 1) {
                    // in this loop we are more efficient by not flushing the buffer.
                    stdout
                        .queue(cursor::MoveTo(x, y))?
                        .queue(style::PrintStyledContent("█".grey()))?;
                }
            }
        }

        Ok(())
    }
}
