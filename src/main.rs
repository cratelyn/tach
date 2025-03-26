//! a compact cpu monitor.

use {
    std::{
        collections::BTreeMap,
        fs::File,
        io::{BufRead, BufReader},
        time::Duration,
    },
    tach::Entry,
};

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    loop {
        print!("{}[2J", 27 as char);
        read()?;
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn read() -> Result<(), Error> {
    let stats = File::open("/proc/stat")
        .map(BufReader::new)
        .expect("file exists")
        .lines();

    let mut entries = {
        let (hint, _) = stats.size_hint();
        Vec::<Entry>::with_capacity(hint)
    };

    for line in stats {
        let line = line?;
        let entry = line.parse::<Entry>()?;
        entries.push(entry);
    }

    let cpus = entries
        .iter()
        .filter_map(|e| match e {
            Entry::Cpu { id, time } => Some((id, time)),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();

    for (cpu, time) in cpus.into_iter() {
        let active = time.active();
        let total = time.total();

        let percent = (active / total) * 100.0;
        assert!(percent >= 0.0);
        assert!(percent <= 100.0);
        let rounded: u32 = percent.round() as u32;
        assert!(rounded <= 100);

        let meter = std::iter::repeat_n('X', rounded as usize)
            .chain(std::iter::repeat(' '))
            .take(100)
            .collect::<String>();
        println!("{cpu:?} {meter}");
    }

    Ok(())
}
