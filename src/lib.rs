//! a compact cpu monitor.

use {
    self::stat::{CpuId, CpuTime, Entry, EntryParseError},
    std::{
        cell::RefCell,
        collections::BTreeMap,
        fmt::Display,
        fs::File,
        io::{self, BufRead, BufReader, BufWriter, Write},
        time::Instant,
    },
};

/// kernel statistics facilities.
///
/// this file provides tools to interact with `/proc/stat`.
mod stat;

/// observes kernel statistics.
pub struct Sentinel {
    /// the last observed snapshot.
    last: RefCell<Snapshot>,
}

/// a comparison of two snapshots.
#[allow(unused, reason = "prototyping")]
struct Delta {
    a: Snapshot,
    b: Snapshot,
}

/// a snapshot of the cpu's statistics.
#[allow(unused, reason = "prototyping")]
pub struct Snapshot {
    cpus: BTreeMap<CpuId, CpuTime>,
    time: Instant,
}

#[derive(Debug)]
pub enum StatReadError {
    Io(io::Error),
    Entry(EntryParseError),
}

/// === impl Sentinel ===

impl Sentinel {
    const STAT: &str = "/proc/stat";

    pub fn start() -> Result<Self, StatReadError> {
        Self::read().map(RefCell::new).map(|last| Self { last })
    }

    pub fn draw<W: Write>(&mut self, out: W) -> Result<(), StatReadError> {
        let mut out = BufWriter::new(out);

        // XXX(kate): todo...
        write!(&mut out, "hello")?;
        write!(&mut out, " world")?;
        write!(&mut out, "\r\n")?;
        out.flush()?;

        Ok(())
    }

    fn read() -> Result<Snapshot, StatReadError> {
        let time = Instant::now();
        let stats = File::open(Self::STAT)
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
            .into_iter()
            .filter_map(|e| match e {
                Entry::Cpu { id, time } => Some((id, time)),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();

        Ok(Snapshot { cpus, time })
    }
}

/// === impl StatReadError ===

impl std::error::Error for StatReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(io) => Some(io),
            Self::Entry(entry) => Some(entry),
        }
    }
}

impl Display for StatReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(io) => f.write_fmt(format_args!("{}", io)),
            Self::Entry(entry) => f.write_fmt(format_args!("{}", entry)),
        }
    }
}

impl From<EntryParseError> for StatReadError {
    fn from(entry: EntryParseError) -> Self {
        Self::Entry(entry)
    }
}

impl From<io::Error> for StatReadError {
    fn from(io: io::Error) -> Self {
        Self::Io(io)
    }
}
