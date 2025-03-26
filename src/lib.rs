//! a compact cpu monitor.

use {
    self::stat::{CpuId, CpuTime, Entry, EntryParseError, UserHz},
    std::{
        cell::RefCell,
        collections::BTreeMap,
        fmt::Display,
        fs::File,
        io::{self, BufRead, BufReader, BufWriter, Write},
        time::{Duration, Instant},
    },
};

/// kernel statistics facilities.
///
/// this file provides tools to interact with `/proc/stat`.
mod stat;

pub struct App {
    sentinel: Sentinel,
}

/// observes kernel statistics.
struct Sentinel {
    /// the last observed snapshot.
    last: RefCell<Snapshot>,
}

/// a comparison of two snapshots.
#[allow(unused, reason = "prototyping")]
struct Delta {
    a: Snapshot,
    b: Snapshot,
}

/// a snapshot of the cpus' statistics at a moment in time.
#[derive(Clone, Debug)]
pub struct Snapshot {
    #[allow(unused, reason = "prototyping")]
    cpus: BTreeMap<CpuId, CpuTime>,
    #[allow(unused, reason = "prototyping")]
    time: Instant,
}

/// a recording of the system's cpu load.
struct Recording {
    /// when the recording began.
    start: Instant,
    /// when the recording ended.
    end: Instant,
    /// how each cpu spent its time.
    cpus: BTreeMap<CpuId, Measurement>,
}

/// a measurement of the difference between two [`CpuTime`]s.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Measurement {
    /// time spent in user mode.
    pub(crate) user: UserHz,
    /// time spent in user mode with low priority (nice).
    pub(crate) nice: UserHz,
    /// time spent in system mode.
    pub(crate) system: UserHz,
    /// time spent in the idle task.
    ///
    /// this value should be USER_HZ times the second entry in the /proc/uptime pseudo-file.
    pub(crate) idle: UserHz,
    /// time waiting for i/o to complete.
    ///
    /// this value is not reliable, for the following reasons:
    ///   *  the cpu will not wait for i/o to complete; iowait is the time that a task is waiting
    ///      for i/o to complete. when a cpu goes into idle state for outstanding task i/o,
    ///      another task will be scheduled on this cpu.
    ///   *  on a multi-core cpu, the task waiting for i/o to complete is not running on any cpu,
    ///      so the iowait of each cpu is difficult to calculate.
    ///   *  the value in this field may decrease in certain conditions.
    pub(crate) iowait: UserHz,
    /// time servicing interrupts.
    pub(crate) irq: UserHz,
    /// time servicing softirqs.
    pub(crate) softirq: UserHz,
    /// stolen time, which is the time spent in other operating systems when running in a
    /// virtualized environment.
    pub(crate) steal: UserHz,
    /// time spent running a virtual cpu for guest operating systems under the control of the linux
    /// kernel.
    pub(crate) guest: UserHz,
    /// time spent running a niced guest (virtual cpu for guest operating systems under the
    /// control of the linux kernel).
    pub(crate) guest_nice: UserHz,
}

#[derive(Debug)]
pub enum StatReadError {
    Io(io::Error),
    Entry(EntryParseError),
}

/// === impl App ===

impl App {
    /// initializes a new application.
    pub fn new() -> Result<Self, StatReadError> {
        Ok(Self {
            sentinel: Sentinel::new()?,
        })
    }

    /// runs the application.
    pub fn run(self) {
        let Self { sentinel } = self;

        loop {
            let recording = sentinel.observe().map(Recording::from).unwrap();
            Self::sleep();
        }
    }

    fn sleep() {
        const INTERVAL: Duration = Duration::from_secs(1);
        std::thread::sleep(INTERVAL);
    }
}

/// === impl Sentinel ===

impl Sentinel {
    const STAT: &str = "/proc/stat";

    pub fn new() -> Result<Self, StatReadError> {
        Self::read().map(RefCell::new).map(|last| Self { last })
    }

    fn observe(&self) -> Result<Delta, StatReadError> {
        let Self { last } = self;

        let new = Self::read()?;
        let prev = last.replace(new.clone());

        Ok(Delta { a: prev, b: new })
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

// === impl Recording ===

impl From<Delta> for Recording {
    fn from(
        Delta {
            a: Snapshot {
                cpus: cpus_a,
                time: time_a,
            },
            b: Snapshot {
                cpus: cpus_b,
                time: time_b,
            },
        }: Delta,
    ) -> Recording {
        assert!(cpus_a.len() == cpus_b.len());
        assert!(time_b > time_a);

        // zip together the two sets of cpu times.
        let mut diff = BTreeMap::new();
        let (mut a_iter, mut b_iter) = (cpus_a.into_iter(), cpus_b.into_iter());
        while let Some((id_a, times_a)) = a_iter.next() {
            let (id_b, times_b) = b_iter.next().unwrap();
            assert!(id_a == id_b);
            let times = Measurement::new(times_a, times_b);
            diff.insert(id_a, times);
        }
        assert!(b_iter.next().is_none());

        Self {
            start: time_a,
            end: time_b,
            cpus: diff,
        }
    }
}

// == impl Measurement ===

impl Measurement {
    pub fn new(a: CpuTime, b: CpuTime) -> Self {
        let a: [_; 10] = a.into();
        let b: [_; 10] = b.into();

        let CpuTime {
            user,
            nice,
            system,
            idle,
            iowait,
            irq,
            softirq,
            steal,
            guest,
            guest_nice,
        } = a
            .iter()
            .enumerate()
            .map(|(i, a_i)| {
                let b_i = b[i];
                b_i - *a_i
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self {
            user,
            nice,
            system,
            idle,
            iowait,
            irq,
            softirq,
            steal,
            guest,
            guest_nice,
        }
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
