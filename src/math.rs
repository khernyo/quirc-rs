use std::{f32, f64};

pub trait RoundToNearestFavorEven {
    // TODO Remove when Rust grows this rounding mode.
    fn round_to_nearest_favor_even(self) -> Self;
}

macro_rules! impl_round_to_nearest_favor_even {
    ($t:tt) => {
        impl RoundToNearestFavorEven for $t {
            fn round_to_nearest_favor_even(self) -> Self {
                let k = 1.0 / $t::EPSILON;
                let a = self.abs();
                if a < k {
                    ((a + k) - k).copysign(self)
                } else {
                    self
                }
            }
        }
    };
}

impl_round_to_nearest_favor_even!(f32);
impl_round_to_nearest_favor_even!(f64);

#[cfg(test)]
mod tests {
    use std::os::raw::c_double;

    use super::*;

    extern "C" {
        fn rint(x: c_double) -> c_double;
    }

    #[test]
    #[allow(clippy::float_cmp)]
    #[allow(clippy::cognitive_complexity)]
    fn test_round_to_nearest_favor_even() {
        assert!(f64::NAN.round_to_nearest_favor_even().is_nan());
        assert_eq!(f64::INFINITY.round_to_nearest_favor_even(), f64::INFINITY);
        assert_eq!(
            f64::NEG_INFINITY.round_to_nearest_favor_even(),
            f64::NEG_INFINITY
        );
        assert_eq!(0f64.round_to_nearest_favor_even(), 0f64);
        assert_eq!((-0f64).round_to_nearest_favor_even(), -0f64);

        assert_eq!(0.5.round_to_nearest_favor_even(), 0f64);
        assert_eq!(1.5.round_to_nearest_favor_even(), 2f64);
        assert_eq!(2.5.round_to_nearest_favor_even(), 2f64);
        assert_eq!(3.5.round_to_nearest_favor_even(), 4f64);
        assert_eq!((-0.5).round_to_nearest_favor_even(), -0f64);
        assert_eq!((-1.5).round_to_nearest_favor_even(), -2f64);
        assert_eq!((-2.5).round_to_nearest_favor_even(), -2f64);
        assert_eq!((-3.5).round_to_nearest_favor_even(), -4f64);

        assert_eq!(0.4.round_to_nearest_favor_even(), 0f64);
        assert_eq!(1.4.round_to_nearest_favor_even(), 1f64);
        assert_eq!(2.4.round_to_nearest_favor_even(), 2f64);
        assert_eq!(3.4.round_to_nearest_favor_even(), 3f64);
        assert_eq!((-0.4).round_to_nearest_favor_even(), -0f64);
        assert_eq!((-1.4).round_to_nearest_favor_even(), -1f64);
        assert_eq!((-2.4).round_to_nearest_favor_even(), -2f64);
        assert_eq!((-3.4).round_to_nearest_favor_even(), -3f64);

        assert_eq!(0.6.round_to_nearest_favor_even(), 1f64);
        assert_eq!(1.6.round_to_nearest_favor_even(), 2f64);
        assert_eq!(2.6.round_to_nearest_favor_even(), 3f64);
        assert_eq!(3.6.round_to_nearest_favor_even(), 4f64);
        assert_eq!((-0.6).round_to_nearest_favor_even(), -1f64);
        assert_eq!((-1.6).round_to_nearest_favor_even(), -2f64);
        assert_eq!((-2.6).round_to_nearest_favor_even(), -3f64);
        assert_eq!((-3.6).round_to_nearest_favor_even(), -4f64);
    }

    #[test]
    #[ignore] // Ignored because it's a long running test
    fn test_round_to_nearest_favor_even_exhaustive() {
        for i in 0..u32::max_value() {
            unsafe {
                let f: f32 = f32::from_bits(i);

                let f_round: f32 = f.round_to_nearest_favor_even();
                let f_round_bits: u32 = std::mem::transmute(f_round);

                let f_rint: f32 = rint(f64::from(f)) as f32;
                let f_rint_bits: u32 = std::mem::transmute(f_rint);

                if f.is_nan() {
                    assert_eq!(f_round.is_nan(), f_rint.is_nan());
                } else {
                    assert_eq!(f_round_bits, f_rint_bits);
                }
            }
        }
    }
}
