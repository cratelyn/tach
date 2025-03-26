use std::{
    ops::{Add, Div, Sub},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserHz(u32);

// === impl UserHz ===

impl UserHz {
    /// the number of clock ticks in a second.
    ///
    /// this can be obtained via `getconf(1)` and `CLK_TCK`, or `sysconf(_SC_CLK_TCK)`. usually,
    /// this is 100Hz, so it is hard-coded for now.
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
        Self(lhs + rhs)
    }
}

impl Sub for UserHz {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let (Self(lhs), Self(rhs)) = (self, rhs);
        Self(lhs - rhs)
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
