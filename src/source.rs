use std::{
    cell::RefCell,
    collections::VecDeque,
    fs::File,
    io::{self, BufReader, Cursor, Read},
    time::Instant,
};

pub use self::{clock::*, stats::*};

mod clock {
    use super::*;

    pub trait Clock {
        fn now(&self) -> Instant;
    }

    #[derive(Default)]
    pub struct SystemClock;

    impl Clock for SystemClock {
        fn now(&self) -> Instant {
            Instant::now()
        }
    }

    /// a mock stat source.
    #[derive(Default)]
    #[allow(dead_code, reason = "this is a testing utility.")]
    pub struct MockStatClock {
        times: RefCell<VecDeque<Instant>>,
    }

    impl Clock for MockStatClock {
        fn now(&self) -> Instant {
            let MockStatClock { times } = self;

            times
                .borrow_mut()
                .pop_front()
                .expect("mock times should not be empty")
        }
    }
}

/// abstracts over providers of statistics.
mod stats {
    use super::*;

    /// a source of kernel statistics.
    pub trait StatsSource {
        /// returns a reader.
        fn open(&self) -> io::Result<impl Read>;
    }

    /// stats backed by `/proc/stat`.
    #[derive(Default)]
    pub struct ProcStatFile;

    /// a mock stat source.
    #[derive(Default)]
    #[allow(dead_code, reason = "this is a testing utility.")]
    pub struct MockStatFile {
        stats: RefCell<VecDeque<String>>,
    }

    // === impl ProcStatFile ===

    impl StatsSource for ProcStatFile {
        fn open(&self) -> io::Result<impl Read> {
            File::open(Self::STAT).map(BufReader::new)
        }
    }

    impl ProcStatFile {
        const STAT: &str = "/proc/stat";
    }

    // === impl MockStatFile ===

    impl StatsSource for MockStatFile {
        fn open(&self) -> io::Result<impl Read> {
            let Self { stats } = self;

            stats
                .borrow_mut()
                .pop_front()
                .map(Cursor::new)
                .map(Ok)
                .expect("mock stats should not be empty")
        }
    }
}
