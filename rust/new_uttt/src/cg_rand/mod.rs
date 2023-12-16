extern crate alloc;

use core::ops::Bound;
use core::ops::RangeBounds;


mod global_rng;

pub use super::*;
pub use global_rng::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Rng(u64);

impl Clone for Rng {
    fn clone(&self) -> Rng {
        Rng::with_seed(self.0)
    }
}

impl Rng {
    #[inline]
    #[cfg(target_pointer_width = "32")]
    fn gen_u32(&mut self) -> u32 {
        self.gen_u64() as u32
    }

    #[inline]
    fn gen_u64(&mut self) -> u64 {
        let s = self.0.wrapping_add(0xA0761D6478BD642F);
        self.0 = s;
        let t = u128::from(s) * u128::from(s ^ 0xE7037ED1A0B428DB);
        (t as u64) ^ (t >> 64) as u64
    }

    #[inline]
    #[cfg(target_pointer_width = "128")]
    fn gen_u128(&mut self) -> u128 {
        (u128::from(self.gen_u64()) << 64) | u128::from(self.gen_u64())
    }

    #[inline]
    #[cfg(target_pointer_width = "32")]
    fn gen_mod_u32(&mut self, n: u32) -> u32 {
        // Adapted from: https://lemire.me/blog/2016/06/30/fast-random-shuffling/
        let mut r = self.gen_u32();
        let mut hi = mul_high_u32(r, n);
        let mut lo = r.wrapping_mul(n);
        if lo < n {
            let t = n.wrapping_neg() % n;
            while lo < t {
                r = self.gen_u32();
                hi = mul_high_u32(r, n);
                lo = r.wrapping_mul(n);
            }
        }
        hi
    }

    #[inline]
    fn gen_mod_u64(&mut self, n: u64) -> u64 {
        // Adapted from: https://lemire.me/blog/2016/06/30/fast-random-shuffling/
        let mut r = self.gen_u64();
        let mut hi = mul_high_u64(r, n);
        let mut lo = r.wrapping_mul(n);
        if lo < n {
            let t = n.wrapping_neg() % n;
            while lo < t {
                r = self.gen_u64();
                hi = mul_high_u64(r, n);
                lo = r.wrapping_mul(n);
            }
        }
        hi
    }

    #[inline]
    #[cfg(target_pointer_width = "128")]
    fn gen_mod_u128(&mut self, n: u128) -> u128 {
        // Adapted from: https://lemire.me/blog/2016/06/30/fast-random-shuffling/
        let mut r = self.gen_u128();
        let mut hi = mul_high_u128(r, n);
        let mut lo = r.wrapping_mul(n);
        if lo < n {
            let t = n.wrapping_neg() % n;
            while lo < t {
                r = self.gen_u128();
                hi = mul_high_u128(r, n);
                lo = r.wrapping_mul(n);
            }
        }
        hi
    }
}

#[inline]
#[cfg(target_pointer_width = "32")]
fn mul_high_u32(a: u32, b: u32) -> u32 {
    (((a as u64) * (b as u64)) >> 32) as u32
}

#[inline]
fn mul_high_u64(a: u64, b: u64) -> u64 {
    (((a as u128) * (b as u128)) >> 64) as u64
}

#[inline]
#[cfg(target_pointer_width = "128")]
fn mul_high_u128(a: u128, b: u128) -> u128 {
    // Adapted from: https://stackoverflow.com/a/28904636
    let a_lo = a as u64 as u128;
    let a_hi = (a >> 64) as u64 as u128;
    let b_lo = b as u64 as u128;
    let b_hi = (b >> 64) as u64 as u128;
    let carry = (a_lo * b_lo) >> 64;
    let carry = ((a_hi * b_lo) as u64 as u128 + (a_lo * b_hi) as u64 as u128 + carry) >> 64;
    a_hi * b_hi + ((a_hi * b_lo) >> 64) + ((a_lo * b_hi) >> 64) + carry
}

macro_rules! rng_integer {
    ($t:tt, $unsigned_t:tt, $gen:tt, $mod:tt, $doc:tt) => {
        #[doc = $doc]
        #[inline]
        pub fn $t(&mut self, range: impl RangeBounds<$t>) -> $t {
            let panic_empty_range = || {
                panic!(
                    "empty range: {:?}..{:?}",
                    range.start_bound(),
                    range.end_bound()
                )
            };

            let low = match range.start_bound() {
                Bound::Unbounded => core::$t::MIN,
                Bound::Included(&x) => x,
                Bound::Excluded(&x) => x.checked_add(1).unwrap_or_else(panic_empty_range),
            };

            let high = match range.end_bound() {
                Bound::Unbounded => core::$t::MAX,
                Bound::Included(&x) => x,
                Bound::Excluded(&x) => x.checked_sub(1).unwrap_or_else(panic_empty_range),
            };

            if low > high {
                panic_empty_range();
            }

            if low == core::$t::MIN && high == core::$t::MAX {
                self.$gen() as $t
            } else {
                let len = high.wrapping_sub(low).wrapping_add(1);
                low.wrapping_add(self.$mod(len as $unsigned_t as _) as $t)
            }
        }
    };
}

impl Rng {
    #[inline]
    #[must_use = "this creates a new instance of `Rng`; if you want to initialize the thread-local generator, use `fastrand::seed()` instead"]
    pub fn with_seed(seed: u64) -> Self {
        let mut rng = Rng(0);

        rng.seed(seed);
        rng
    }

    #[inline]
    #[must_use = "this creates a new instance of `Rng`"]
    pub fn fork(&mut self) -> Self {
        Rng::with_seed(self.gen_u64())
    }    

    #[inline]
    pub fn seed(&mut self, seed: u64) {
        self.0 = seed;
    }

    #[inline]
    pub fn get_seed(&self) -> u64 {
        self.0
    }

    #[inline]
    pub fn choice<I>(&mut self, iter: I) -> Option<I::Item>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
    {
        let mut iter = iter.into_iter();

        // Get the item at a random index.
        let len = iter.len();
        if len == 0 {
            return None;
        }
        let index = self.usize(0..len);

        iter.nth(index)
    }

    #[cfg(target_pointer_width = "32")]
    rng_integer!(
        usize,
        usize,
        gen_u32,
        gen_mod_u32,
        "Generates a random `usize` in the given range."
    );
    #[cfg(target_pointer_width = "64")]
    rng_integer!(
        usize,
        usize,
        gen_u64,
        gen_mod_u64,
        "Generates a random `usize` in the given range."
    );
    #[cfg(target_pointer_width = "128")]
    rng_integer!(
        usize,
        usize,
        gen_u128,
        gen_mod_u128,
        "Generates a random `usize` in the given range."
    );
}