#![allow(dead_code, reason = "TODO(kate): refactoring display")]

use std::{
    fmt::Display,
    io::{self, Write},
};

pub struct Meter {
    pub value: f64,
    pub width: usize,
}

/// a reading is a list of cells.
// xxx rename this to meter
pub struct Reading {
    cells: Vec<Cell>,
}

/// a cell in a meter.
#[derive(Debug, Clone)]
enum Cell {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

/// === impl Meter ===

impl Meter {
    // XXX: a simple, hacky meter.
    pub fn draw(&self, writer: &mut impl Write) -> io::Result<()> {
        let Reading { cells } = placeholder(self.value, self.width);
        let reading = Reading {
            cells: middle_fill(cells.into_iter()),
        };
        writer.write(reading.to_string().as_bytes())?;
        Ok(())
    }
}

fn placeholder(percentage: f64, width: usize) -> Reading {
    assert!(percentage >= 0.0);
    assert!(percentage <= 1.0);
    assert!(width > 0);

    // how many dots should we display?
    let amount = {
        let resolution = Cell::RESOLUTION as usize * width;
        let amount = resolution as f64 * percentage;
        amount.round() as u8
    };

    let filled = (amount / Cell::RESOLUTION).into();
    let rem = Cell::try_from(amount % Cell::RESOLUTION).expect("remainder should not panic");
    assert!(filled <= width);

    let cells = std::iter::repeat_n(Cell::Eight, filled)
        .chain(std::iter::once(rem))
        .chain(std::iter::repeat(Cell::Zero))
        .take(width)
        .collect();

    Reading { cells }
}

fn middle_fill(mut cells: impl Iterator<Item = Cell>) -> Vec<Cell> {
    use std::collections::VecDeque;
    let mut new = VecDeque::with_capacity(cells.size_hint().0);
    let mut flip = false;

    while let Some(next) = cells.next() {
        if flip {
            new.push_front(next);
        } else {
            new.push_back(next);
        }
        flip = !flip;
    }

    new.into()
}

impl Display for Reading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { cells } = self;
        for c in cells.iter().map(Cell::as_char) {
            use std::fmt::Write;
            f.write_char(c)?;
        }
        Ok(())
    }
}

impl TryFrom<u8> for Cell {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Cell::*;
        Ok(match value {
            0 => Zero,
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            5 => Five,
            6 => Six,
            7 => Seven,
            8 => Eight,
            other => return Err(other),
        })
    }
}

impl Cell {
    /// the "resolution" of a cell.
    ///
    /// zero to eight dots can be shown in a cell.
    const RESOLUTION: u8 = 8;

    fn as_char(&self) -> char {
        use {Cell::*, chars::*};
        match self {
            Zero => placeholder::ZERO,
            One => placeholder::ONE,
            Two => placeholder::TWO,
            Three => placeholder::THREE,
            Four => placeholder::FOUR,
            Five => placeholder::FIVE,
            Six => placeholder::SIX,
            Seven => placeholder::SEVEN,
            Eight => placeholder::EIGHT,
        }
    }
}

/// characters for drawing a [`Meter`].
///
/// [unicode]: https://www.unicode.org/charts/PDF/U2800.pdf
#[rustfmt::skip]
mod chars {
    pub mod placeholder {
        pub const ZERO: char = '\u{2800}';  // `⠀`
        pub const ONE: char = '\u{2840}';   // `⡀`
        pub const TWO: char = '\u{28C0}';   // `⣀`
        pub const THREE: char = '\u{28C4}'; // `⣄`
        pub const FOUR: char = '\u{28E4}';  // `⣤`
        pub const FIVE: char = '\u{28E6}';  // `⣦`
        pub const SIX: char = '\u{28F6}';   // `⣶`
        pub const SEVEN: char = '\u{28F7}'; // `⣷`
        pub const EIGHT: char = '\u{28FF}'; // `⣿`
    }
}

#[cfg(test)]
mod placeholder_tests {
    use super::*;

    #[test]
    fn zero_width_one() {
        let (percentage, width) = (0.0, 1);
        let reading = placeholder(percentage, width);
        let s = reading.to_string();
        assert_eq!(s, "⠀");
    }

    #[test]
    fn zero_width_eight() {
        let (percentage, width) = (0.0, 8);
        let reading = placeholder(percentage, width);
        let s = reading.to_string();
        assert_eq!(s, "⠀⠀⠀⠀⠀⠀⠀⠀");
    }

    #[test]
    fn zero_width_ten() {
        let (percentage, width) = (0.0, 10);
        let reading = placeholder(percentage, width);
        let s = reading.to_string();
        assert_eq!(s, "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀");
    }

    #[test]
    fn one_eighth_width_one() {
        let (percentage, width) = (0.125, 1);
        let reading = placeholder(percentage, width);
        let s = reading.to_string();
        assert_eq!(s, "⡀");
    }

    #[test]
    fn one_quarter_width_one() {
        let (percentage, width) = (0.25, 1);
        let reading = placeholder(percentage, width);
        let s = reading.to_string();
        assert_eq!(s, "⣀");
    }

    #[test]
    fn one_quarter_width_eight() {
        let (percentage, width) = (0.25, 8);
        let reading = placeholder(percentage, width);
        let s = reading.to_string();
        assert_eq!(s, "⣿⣿⠀⠀⠀⠀⠀⠀");
    }

    #[test]
    fn three_eighths_width_one() {
        let (percentage, width) = (0.325, 1);
        let reading = placeholder(percentage, width);
        let s = reading.to_string();
        assert_eq!(s, "⣄");
    }

    #[test]
    fn half_width_eight() {
        let (percentage, width) = (0.50, 8);
        let reading = placeholder(percentage, width);
        let s = reading.to_string();
        assert_eq!(s, "⣿⣿⣿⣿⠀⠀⠀⠀");
    }

    #[test]
    fn three_quarter_width_eight() {
        let (percentage, width) = (0.75, 8);
        let reading = placeholder(percentage, width);
        let s = reading.to_string();
        assert_eq!(s, "⣿⣿⣿⣿⣿⣿⠀⠀");
    }

    #[test]
    fn seven_eighths_width_one() {
        let (percentage, width) = (0.875, 1);
        let reading = placeholder(percentage, width);
        let s = reading.to_string();
        assert_eq!(s, "⣷");
    }

    #[test]
    fn full_width_eight() {
        let (percentage, width) = (1.00, 8);
        let reading = placeholder(percentage, width);
        let s = reading.to_string();
        assert_eq!(s, "⣿⣿⣿⣿⣿⣿⣿⣿");
    }
}
