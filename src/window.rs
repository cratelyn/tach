//! the tui window for the application.

#![allow(unused, reason = "XXX")]

use {
    super::*,
    crossterm::{
        ExecutableCommand, QueueableCommand, cursor,
        style::{self, Print, Stylize},
        terminal,
    },
    std::collections::VecDeque,
};

#[derive(Clone, Copy, Debug)]
pub struct Width(pub u16);

#[derive(Clone, Copy, Debug)]
pub struct Height(pub u16);

#[derive(Clone, Copy, Debug)]
pub struct Size {
    pub width: Width,
    pub height: Height,
}

#[derive(Clone, Copy, Debug)]
pub struct X(u16);

#[derive(Clone, Copy, Debug)]
pub struct Y(u16);

#[derive(Clone, Copy, Debug)]
pub struct Coordinate {
    x: X,
    y: Y,
}

// === impl App ===

impl App {
    /// begins running the tui in the current terminal.
    pub fn tui(self) -> Result<(), crate::Error> {
        let Self {
            mut sentinel,
            num_cpus,
            size,
        } = self;

        // clear the screen before we draw anything to it.
        Self::clear()?;

        let mut recordings = VecDeque::<Recording>::new();
        loop {
            Self::border(size)?;

            if let Some(Recording {
                start: _,
                end: _,
                system: _,
                cpus,
            }) = sentinel.observe()?
            {
                /*for (cpu, _) in cpus.iter() {
                    io::stdout()
                        .queue(cursor::MoveTo(((cpu.as_u16() * 10) + 2) as u16, 2))?
                        .queue(style::PrintStyledContent(
                            format!("{}", cpu.as_u16()).grey(),
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
                }*/
            }

            // hide the cursor, flush the output, and then sleep until the next tick.
            io::stdout().queue(cursor::Hide)?.flush()?;
            Self::sleep();
        }
    }

    /// clears the screen.
    fn clear() -> Result<(), io::Error> {
        io::stdout()
            .execute(terminal::Clear(terminal::ClearType::All))
            .map(drop)
    }

    /// draws a character at the specified position.
    ///
    /// the origin (0, 0) is at the top-left of the screen.
    fn draw_char(Coordinate { x: X(x), y: Y(y) }: Coordinate, c: char) -> Result<(), io::Error> {
        let mut stdout = io::stdout();
        let move_to = cursor::MoveTo(x, y);
        let print = Print(c);

        stdout.queue(move_to)?.queue(print)?;
        Ok(())
    }

    /// draws borders around the corner of the screen.
    fn border(size: Size) -> Result<(), io::Error> {
        use crate::chars::box_drawing::light::{
            ARC_DOWN_AND_LEFT, ARC_DOWN_AND_RIGHT, ARC_UP_AND_LEFT, ARC_UP_AND_RIGHT, HORIZONTAL,
            VERTICAL,
        };

        let mut stdout = io::stdout();

        let corners = {
            let top_left = (size.top_left(), ARC_DOWN_AND_RIGHT);
            let top_right = (size.top_right(), ARC_DOWN_AND_LEFT);
            let bot_left = (size.bottom_left(), ARC_UP_AND_RIGHT);
            let bot_right = (size.bottom_right(), ARC_UP_AND_LEFT);
            [top_left, top_right, bot_left, bot_right]
        };

        let lhs = size.left_side().map(|pos| (pos, VERTICAL));
        let rhs = size.right_side().map(|pos| (pos, VERTICAL));
        let ths = size.top_side().map(|pos| (pos, HORIZONTAL));
        let bhs = size.bottom_side().map(|pos| (pos, HORIZONTAL));

        corners
            .into_iter()
            .chain(lhs)
            .chain(rhs)
            .chain(ths)
            .chain(bhs)
            .try_for_each(|(pos, c)| Self::draw_char(pos, c))?;

        Ok(())
    }
}

// === impl Size ===

impl Size {
    fn left(&self) -> X {
        X(0)
    }

    fn right(&self) -> X {
        let Self {
            width: Width(cols),
            height: _,
        } = self;

        X(cols - 1)
    }

    fn top(&self) -> Y {
        Y(0)
    }

    fn bottom(&self) -> Y {
        let Self {
            width: _,
            height: Height(rows),
        } = self;

        Y(rows - 1)
    }

    pub fn top_left(&self) -> Coordinate {
        let x = self.left();
        let y = self.top();
        Coordinate { x, y }
    }

    pub fn top_right(&self) -> Coordinate {
        let x = self.right();
        let y = self.top();
        Coordinate { x, y }
    }

    pub fn bottom_left(&self) -> Coordinate {
        let x = self.left();
        let y = self.bottom();
        Coordinate { x, y }
    }

    pub fn bottom_right(&self) -> Coordinate {
        let x = self.right();
        let y = self.bottom();
        Coordinate { x, y }
    }

    fn top_side(&self) -> impl Iterator<Item = Coordinate> {
        let Self {
            width: Width(rows), ..
        } = self;

        let start = 1;
        let end = (rows - 1);
        let y = self.top();

        (start..end)
            .map(X)
            .map(move |x| (x, y))
            .map(Coordinate::from)
    }

    fn bottom_side(&self) -> impl Iterator<Item = Coordinate> {
        let Self {
            width: Width(rows), ..
        } = self;

        let start = 1;
        let end = (rows - 1);
        let y = self.bottom();

        (start..end)
            .map(X)
            .map(move |x| (x, y))
            .map(Coordinate::from)
    }

    fn left_side(&self) -> impl Iterator<Item = Coordinate> {
        let Self {
            height: Height(cols),
            ..
        } = self;

        let start = 1;
        let end = (cols - 1);
        let x = self.left();

        (start..end)
            .map(Y)
            .map(move |y| (x, y))
            .map(Coordinate::from)
    }

    fn right_side(&self) -> impl Iterator<Item = Coordinate> {
        let Self {
            height: Height(cols),
            ..
        } = self;

        let start = 1;
        let end = (cols - 1);
        let x = self.right();

        (start..end)
            .map(Y)
            .map(move |y| (x, y))
            .map(Coordinate::from)
    }
}

// === impl Coordinate ===

impl From<(u16, u16)> for Coordinate {
    fn from((x, y): (u16, u16)) -> Self {
        let x = X(x);
        let y = Y(y);
        Self::from((x, y))
    }
}

impl From<(X, Y)> for Coordinate {
    fn from((x, y): (X, Y)) -> Self {
        Self { x, y }
    }
}
