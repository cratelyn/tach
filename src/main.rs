//! a compact cpu monitor.

use std::{io::stdout, thread::sleep, time::Duration};

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    let stdout = stdout();
    let mut sentinel = tach::Sentinel::start()?;
    let clear = || print!("{}[2J", 27 as char);
    let sleep = || sleep(Duration::from_secs(1));

    loop {
        clear();
        sentinel.draw(stdout.lock())?;
        sleep();
    }
}
