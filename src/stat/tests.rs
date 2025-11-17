use super::*;

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

    #[test]
    #[ignore = "TODO"]
    fn big() {
        let entry = "cpu  5000000000 5000000000 5000000000 5000000000 5000000000 5000000000 5000000000 0 0 0"
                .parse::<Entry>()
                .unwrap();
        assert_eq!(entry, Entry::SoftIrq);
    }
}

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
