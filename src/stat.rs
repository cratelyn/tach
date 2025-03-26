use std::{
    fmt,
    ops::{Add, Deref, Div, Not},
    str::FromStr,
};

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

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CpuId(u8);

#[derive(Debug, Eq, PartialEq)]
pub struct CpuTime {
    /// time spent in user mode.
    user: UserHz,
    /// time spent in user mode with low priority (nice).
    nice: UserHz,
    /// time spent in system mode.
    system: UserHz,
    /// time spent in the idle task.
    ///
    /// this value should be USER_HZ times the second entry in the /proc/uptime pseudo-file.
    idle: UserHz,
    /// time waiting for i/o to complete.
    ///
    /// this value is not reliable, for the following reasons:
    ///   *  the cpu will not wait for i/o to complete; iowait is the time that a task is waiting
    ///      for i/o to complete. when a cpu goes into idle state for outstanding task i/o,
    ///      another task will be scheduled on this cpu.
    ///   *  on a multi-core cpu, the task waiting for i/o to complete is not running on any cpu,
    ///      so the iowait of each cpu is difficult to calculate.
    ///   *  the value in this field may decrease in certain conditions.
    iowait: UserHz,
    /// time servicing interrupts.
    irq: UserHz,
    /// time servicing softirqs.
    softirq: UserHz,
    /// stolen time, which is the time spent in other operating systems when running in a
    /// virtualized environment.
    steal: UserHz,
    /// time spent running a virtual cpu for guest operating systems under the control of the linux
    /// kernel.
    guest: UserHz,
    /// time spent running a niced guest (virtual cpu for guest operating systems under the
    /// control of the linux kernel).
    guest_nice: UserHz,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserHz(u32);

#[derive(Debug, Eq, PartialEq)]
pub enum EntryParseError {
    UnrecognizedEntry { kind: String },
    CpuIdParse(<u8 as FromStr>::Err),
    UserHzParse(<UserHz as FromStr>::Err),
    CpuTime,
}

enum Either<'a> {
    Cpu(&'a str),
    Entry(Entry),
}

// === impl UserHz ===

impl UserHz {
    /// the number of clock ticks in a second.
    ///
    /// this can be obtained via `getconf(1)` and `CLK_TCK`, or `sysconf(_SC_CLK_TCK)`. usually, this
    /// is 100Hz, so it is hard-coded for now.
    #[allow(unused, reason = "prototyping")]
    const FREQ: u8 = 100;
}

impl FromStr for UserHz {
    type Err = <u128 as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl Add for UserHz {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let (Self(lhs), Self(rhs)) = (self, rhs);
        UserHz(lhs + rhs)
    }
}

impl Div for UserHz {
    type Output = f64;
    fn div(self, rhs: Self) -> Self::Output {
        let to_float = |Self(hz)| -> f64 { hz.try_into().unwrap() };
        let (lhs, rhs) = (to_float(self), to_float(rhs));

        lhs / rhs
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

// === impl CpuTime ===

impl CpuTime {
    pub fn active(&self) -> UserHz {
        let Self {
            user,
            nice,
            system,
            iowait,
            irq,
            softirq,
            steal,
            guest,
            guest_nice,
            idle: _, // do not count idle time...
        } = *self;

        user + nice + system + iowait + irq + softirq + steal + guest + guest_nice
    }

    pub fn total(&self) -> UserHz {
        let Self {
            user,
            nice,
            system,
            iowait,
            irq,
            softirq,
            steal,
            guest,
            guest_nice,
            idle,
        } = *self;

        user + nice + system + iowait + irq + softirq + steal + guest + guest_nice + idle
    }
}

impl TryFrom<Vec<UserHz>> for CpuTime {
    type Error = EntryParseError;
    fn try_from(times: Vec<UserHz>) -> Result<Self, Self::Error> {
        <_ as TryInto<[_; 10]>>::try_into(times)
            .map(Self::from)
            .map_err(|_| EntryParseError::CpuTime)
    }
}

impl From<[UserHz; 10]> for CpuTime {
    fn from(
        [
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
        ]: [UserHz; 10],
    ) -> Self {
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

// === unit tests ===

#[cfg(test)]
mod entry_parse_tests {
    use super::*;

    // two examples provided in the `proc_stat(5)` man page.
    const EXAMPLE_1: &str = "cpu 10132153 290696 3084719 46828483 16683 0 25195 0 175628 0";
    const EXAMPLE_2: &str = "cpu0 1393280 32966 572056 13343292 6130 0 17875 0 23933 0";

    #[test]
    fn example_1() {
        let _ = EXAMPLE_1.parse::<Entry>().unwrap();
    }

    #[test]
    fn example_2() {
        let _ = EXAMPLE_2.parse::<Entry>().unwrap();
    }

    #[test]
    fn example_3() {
        const EXAMPLE_3: &str = "cpu  10132153 290696 3084719 46828483 16683 0 25195 0 175628 0";
        let _ = EXAMPLE_3.parse::<Entry>().unwrap();
    }

    #[test]
    fn bad_cpu_id() {
        let err = "cpuA 0 0 0 0 0 0 0 0 0 0".parse::<Entry>().unwrap_err();
        assert!(matches!(err, EntryParseError::CpuIdParse(_)));
    }

    #[test]
    fn bad_entry_kind() {
        const ENTRY: &str = "wrong 0 0 0 0 0 0 0 0 0 0";
        let err = ENTRY.parse::<Entry>().unwrap_err();
        match err {
            EntryParseError::UnrecognizedEntry { kind } if kind == "wrong" => {}
            _other => panic!(),
        }
    }

    /// parse a cpu entry that is missing one of its times.
    #[test]
    fn missing_time() {
        const ENTRY: &str = "cpu 10132153 290696 3084719 46828483 16683 0 25195 0 175628";
        let err = ENTRY.parse::<Entry>().unwrap_err();
        assert_eq!(err, EntryParseError::CpuTime);
    }

    /// parse a cpu entry that has one too many times..
    #[test]
    fn extra_time() {
        const ENTRY: &str = "cpu 10132153 290696 3084719 46828483 16683 0 25195 0 175628 0 0";
        let err = ENTRY.parse::<Entry>().unwrap_err();
        assert_eq!(err, EntryParseError::CpuTime);
    }

    #[test]
    fn page() {
        let entry = "page 5741 1808".parse::<Entry>().unwrap();
        assert_eq!(entry, Entry::Page);
    }

    #[test]
    fn swap() {
        let entry = "swap 1 0".parse::<Entry>().unwrap();
        assert_eq!(entry, Entry::Swap);
    }

    #[test]
    fn intr() {
        let entry = "intr 1462898".parse::<Entry>().unwrap();
        assert_eq!(entry, Entry::Intr);
    }

    #[test]
    fn btime() {
        let entry = "btime 769041601".parse::<Entry>().unwrap();
        assert_eq!(entry, Entry::Btime);
    }

    #[test]
    fn processes() {
        let entry = "processes 86031".parse::<Entry>().unwrap();
        assert_eq!(entry, Entry::Processes);
    }

    #[test]
    fn procs_running() {
        let entry = "procs_running 6".parse::<Entry>().unwrap();
        assert_eq!(entry, Entry::ProcsRunning);
    }

    #[test]
    fn procs_blocked() {
        let entry = "procs_blocked 2".parse::<Entry>().unwrap();
        assert_eq!(entry, Entry::ProcsBlocked);
    }

    #[test]
    fn softirq() {
        let entry =
            "softirq 229245889 94 60001584 13619 5175704 2471304 28 51212741 59130143 0 51240672"
                .parse::<Entry>()
                .unwrap();
        assert_eq!(entry, Entry::SoftIrq);
    }
}

#[cfg(test)]
mod parse_cpu_id_tests {
    use super::*;

    #[test]
    fn all() {
        assert_eq!(Entry::parse_cpu_id("cpu"), Ok(None));
    }

    #[test]
    fn one() {
        assert_eq!(Entry::parse_cpu_id("cpu1"), Ok(Some(CpuId(1))));
    }

    #[test]
    fn two() {
        assert_eq!(Entry::parse_cpu_id("cpu2"), Ok(Some(CpuId(2))));
    }

    #[test]
    fn a() {
        assert!(matches!(
            Entry::parse_cpu_id("cpua"),
            Err(EntryParseError::CpuIdParse(_))
        ));
    }
}
