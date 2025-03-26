use std::{
    io::{self, Write},
    iter::{once, repeat, repeat_n},
};

pub struct Meter {
    pub name: String,
    pub value: usize,
    pub width: usize,
}

/// === impl Meter ===

impl Meter {
    // XXX: a simple, hacky meter.
    pub fn draw(&self, writer: &mut impl Write) -> io::Result<()> {
        const ACTIVE: char = '█';
        const IDLE: char = ' ';
        const BORDER_L: char = '[';
        const BORDER_R: char = ']';

        let Self { name, value, width } = self;
        assert!(value <= width);

        // print the label.
        let label = format!("{name}: ").into_bytes();
        writer.write(&label).map(|_| ())?;

        // print the meter.
        let meter = {
            let active = repeat_n(ACTIVE, *value);
            let idle = repeat(IDLE);
            active.chain(idle).take(*width)
        };
        let meter = once(BORDER_L).chain(meter).chain(once(BORDER_R));
        let bytes = meter.collect::<String>().into_bytes();
        writer.write(&bytes).map(|_| ())?;

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {}
