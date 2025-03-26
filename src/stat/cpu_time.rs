use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
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

/// a measurement of the difference between two [`CpuTime`]s.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Measurement {
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

    /// returns the percentage of active cpu time.
    pub fn percentage(&self) -> u8 {
        let active = self.active();
        let total = self.total();

        // calculate a percentage.
        let percent = (active / total) * 100.0;
        assert!(percent >= 0.0);
        assert!(percent <= 100.0);

        // round to the nearest percentage point.
        let rounded: u8 = percent.round() as u8;
        assert!(rounded <= 100);

        rounded
    }

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

// === impl CpuTime ===

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

impl Into<[UserHz; 10]> for CpuTime {
    fn into(self) -> [UserHz; 10] {
        let Self {
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
        } = self;

        [
            user, nice, system, idle, iowait, irq, softirq, steal, guest, guest_nice,
        ]
    }
}
