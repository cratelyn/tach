use {
    crate::source::{Clock, StatsSource},
    std::{
        collections::BTreeMap,
        fmt::{self, Display},
        io::{self, BufRead, BufReader},
        ops::{Deref, Not},
        str::FromStr,
        time::Instant,
    },
};

pub use self::{
    cpu_time::{CpuTime, Measurement},
    user_hz::UserHz,
};

mod cpu_time;
mod user_hz;

#[cfg(test)]
mod tests;

/// a snapshot of the cpus' statistics at a moment in time.
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub system: CpuTime,
    pub cpus: BTreeMap<CpuId, CpuTime>,
    pub time: Instant,
}

/// an entry in the `/proc/stat` kernel statistics table.
///
/// see `proc_stat(5)` for more information.
#[derive(Debug, Eq, PartialEq)]
pub enum Entry {
    /// the amount of time that the system ("cpu" line) spent in various states.
    AllCpu {
        time: CpuTime,
    },
    /// the amount of time that a specific cpu ("cpuN" line) spent in various states.
    Cpu {
        id: CpuId,
        time: CpuTime,
    },
    /// the number of pages the system paged in and the number that were paged out (from disk).
    Page,
    /// the number of swap pages that have been brought in and out.
    Swap,
    /// this line shows counts of interrupts serviced since boot time.
    Intr,
    DiskIo,
    /// the number of context switches that the system underwent.
    Ctxt,
    Btime,
    /// the number of forks since boot.
    Processes,
    /// the number of processes in runnable state.  (linux 2.5.45 onward.)
    ProcsRunning,
    /// the number of processes blocked waiting for i/o to complete.
    ProcsBlocked,
    /// this line shows the number of softirq for all cpus.
    SoftIrq,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CpuId(u8);

#[derive(Debug, Eq, PartialEq)]
pub enum EntryParseError {
    UnrecognizedEntry { kind: String },
    CpuIdParse(<u8 as FromStr>::Err),
    UserHzParse(<UserHz as FromStr>::Err),
    CpuTime,
}

#[derive(Debug)]
pub enum StatReadError {
    Io(io::Error),
    Entry(EntryParseError),
}

enum Either<'a> {
    Cpu(&'a str),
    Entry(Entry),
}

// === impl Snapshot ===

impl Snapshot {
    /// uses the given source to parse a snapshot of the cpu statistics.
    pub(super) fn read(
        stats: &impl StatsSource,
        clock: &impl Clock,
    ) -> Result<Snapshot, StatReadError> {
        let time = clock.now();
        let stats = {
            let reader = stats.open().map_err(StatReadError::Io)?;
            BufReader::new(reader).lines().collect::<Vec<_>>()
        };

        let mut entries = {
            let len = stats.len();
            Vec::<Entry>::with_capacity(len)
        };

        for line in stats {
            let line = line?;
            let entry = line.parse::<Entry>()?;
            entries.push(entry);
        }

        let (system, cpus) =
            entries
                .into_iter()
                .fold((None, BTreeMap::default()), |(mut sys, mut cpus), entry| {
                    match entry {
                        Entry::Cpu { id, time } => cpus.insert(id, time),
                        Entry::AllCpu { time } => sys.replace(time),
                        _ => None,
                    };
                    (sys, cpus)
                });

        let system = system.expect("system cpu statistic should exist");

        Ok(Snapshot { system, cpus, time })
    }
}

// === impl Entry ===

impl FromStr for Entry {
    type Err = EntryParseError;
    fn from_str(entry: &str) -> Result<Self, Self::Err> {
        let tokens = entry
            .split(' ')
            .filter(|t| t.is_empty().not())
            .collect::<Vec<_>>();
        let [kind, tokens @ ..] = tokens.as_slice() else {
            todo!()
        };

        let id = match Self::parse_entry_kind(kind) {
            Either::Cpu(cpu) => Self::parse_cpu_id(cpu)?,
            Either::Entry(entry) => return Ok(entry),
        };

        let time = tokens
            .into_iter()
            .map(Deref::deref)
            .map(str::parse::<UserHz>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(EntryParseError::UserHzParse)
            .and_then(CpuTime::try_from)?;

        Ok(if let Some(id) = id {
            Self::Cpu { id, time }
        } else {
            Self::AllCpu { time }
        })
    }
}

impl Entry {
    fn parse_entry_kind(kind: &str) -> Either {
        use Entry::*;

        match kind {
            "page" => Either::Entry(Page),
            "swap" => Either::Entry(Swap),
            "intr" => Either::Entry(Intr),
            "disk_io" => Either::Entry(DiskIo),
            "ctxt" => Either::Entry(Ctxt),
            "btime" => Either::Entry(Btime),
            "processes" => Either::Entry(Processes),
            "procs_running" => Either::Entry(ProcsRunning),
            "procs_blocked" => Either::Entry(ProcsBlocked),
            "softirq" => Either::Entry(SoftIrq),
            cpu => Either::Cpu(cpu),
        }
    }

    fn parse_cpu_id(token: &str) -> Result<Option<CpuId>, EntryParseError> {
        use EntryParseError::{CpuIdParse, UnrecognizedEntry};

        // strip the token of its "cpu" prefix.
        let suffix = token.strip_prefix("cpu").ok_or_else(|| UnrecognizedEntry {
            kind: token.to_owned(),
        })?;

        // if there is no suffix, return `None`.
        if suffix.is_empty() {
            return Ok(None);
        }

        // parse the id into an integer.
        suffix
            .parse::<u8>()
            .map(CpuId)
            .map(Some)
            .map_err(CpuIdParse)
    }
}

// === impl StatReadError ===

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

// === impl EntryParseError ===

impl fmt::Display for EntryParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use EntryParseError::*;
        match self {
            UnrecognizedEntry { kind } => {
                f.write_fmt(format_args!("unrecognized entry kind: {kind}"))
            }
            CpuIdParse(error) => f.write_fmt(format_args!("invalid cpu id: {error}")),
            UserHzParse(error) => f.write_fmt(format_args!("invalid time value: {error}")),
            CpuTime => f.write_str("some other error"), // XXX(kate)
        }
    }
}

impl std::error::Error for EntryParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use EntryParseError::*;

        match self {
            CpuIdParse(error) => Some(error),
            UserHzParse(error) => Some(error),
            UnrecognizedEntry { kind: _ } | CpuTime => None,
        }
    }
}
