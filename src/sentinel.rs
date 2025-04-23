use {
    crate::{
        source::{Clock, ProcStatFile, StatsSource, SystemClock},
        stat::{CpuId, Measurement, Snapshot, StatReadError},
    },
    std::{collections::BTreeMap, time::Instant},
};

/// observes kernel statistics.
pub struct Sentinel<C = SystemClock, S = ProcStatFile> {
    inner: Inner<C, S>,
}

enum Inner<C, S> {
    Initialized {
        /// the clock being used to measure time.
        clock: C,
        /// the underlying source of kernel statistics.
        source: S,
    },
    Running {
        /// the clock being used to measure time.
        clock: C,
        /// the underlying source of kernel statistics.
        source: S,
        /// the last observed snapshot.
        last: Snapshot,
    },
}

/// a recording of the system's cpu load.
#[derive(Clone, Debug)]
pub struct Recording {
    /// when the recording began.
    #[allow(dead_code)]
    pub start: Instant,
    /// when the recording ended.
    #[allow(dead_code)]
    pub end: Instant,
    /// how the system cpus spent their time, in aggregate.
    pub system: Measurement,
    /// how each cpu spent its time.
    pub cpus: BTreeMap<CpuId, Measurement>,
}

/// === impl Sentinel ===

impl<S: Default, C: Default> Sentinel<C, S> {
    /// creates a new [`Sentinel`].
    pub fn new() -> Self {
        Self {
            inner: Inner::Initialized {
                clock: C::default(),
                source: S::default(),
            },
        }
    }
}

impl<S, C> Sentinel<C, S>
where
    S: StatsSource + Default,
    C: Clock + Default,
{
    /// returns a [`Recording`] of cpu time since this was last called.
    ///
    /// NB: by virtue of this being a comparison to the previous reading, this will return
    /// `Ok(None)` the first time it is called.
    pub fn observe(&mut self) -> Result<Option<Recording>, StatReadError> {
        let Self { inner } = self;

        match inner {
            Inner::Initialized { clock, source } => {
                let clock = std::mem::take(clock);
                let source = std::mem::take(source);
                let last = Snapshot::read(&source, &clock)?;
                *inner = Inner::Running {
                    clock,
                    source,
                    last,
                };
                Ok(None)
            }
            Inner::Running {
                clock,
                source: stats,
                last,
            } => {
                let new = Snapshot::read(stats, clock)?;
                let prev = std::mem::replace(last, new.clone());
                let recording = Recording::new(prev, new);
                Ok(Some(recording))
            }
        }
    }
}

// === impl Recording ===

impl Recording {
    fn new(
        Snapshot {
            system: system_a,
            cpus: cpus_a,
            time: time_a,
        }: Snapshot,
        Snapshot {
            system: system_b,
            cpus: cpus_b,
            time: time_b,
        }: Snapshot,
    ) -> Recording {
        assert!(cpus_a.len() == cpus_b.len());
        assert!(time_b > time_a);

        let system = Measurement::new(system_a, system_b);

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
            system,
            cpus: diff,
        }
    }
}
