use super::cg_rand::Rng;

use std::cell::Cell;
use std::ops::RangeBounds;

const DEFAULT_RNG_SEED: u64 = 0xef6f79ed30ba75a;

impl Default for Rng {
    #[inline]
    fn default() -> Rng {
        Rng::new()
    }
}

impl Rng {
    #[inline]
    pub fn new() -> Rng {
        try_with_rng(Rng::fork).unwrap_or_else(|_| Rng::with_seed(0x4d595df4d0f33173))
    }
}

thread_local! {
    static RNG: Cell<Rng> = Cell::new(Rng(random_seed().unwrap_or(DEFAULT_RNG_SEED)));
}

#[inline]
fn with_rng<R>(f: impl FnOnce(&mut Rng) -> R) -> R {
    RNG.with(|rng| {
        let current = rng.replace(Rng(0));

        let mut restore = RestoreOnDrop { rng, current };

        f(&mut restore.current)
    })
}

#[inline]
fn try_with_rng<R>(f: impl FnOnce(&mut Rng) -> R) -> Result<R, std::thread::AccessError> {
    RNG.try_with(|rng| {
        let current = rng.replace(Rng(0));

        let mut restore = RestoreOnDrop { rng, current };

        f(&mut restore.current)
    })
}

struct RestoreOnDrop<'a> {
    rng: &'a Cell<Rng>,
    current: Rng,
}

impl Drop for RestoreOnDrop<'_> {
    fn drop(&mut self) {
        self.rng.set(Rng(self.current.0));
    }
}

#[inline]
pub fn seed(seed: u64) {
    with_rng(|r| r.seed(seed));
}

#[inline]
pub fn get_seed() -> u64 {
    with_rng(|r| r.get_seed())
}

#[inline]
pub fn choice<I>(iter: I) -> Option<I::Item>
where
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator,
{
    with_rng(|r| r.choice(iter))
}

#[inline]
pub fn usize(range: impl RangeBounds<usize>) -> usize {
    with_rng(|r| r.usize(range))
}

fn random_seed() -> Option<u64> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::thread;
    use std::time::Instant;

    let mut hasher = DefaultHasher::new();
    Instant::now().hash(&mut hasher);
    thread::current().id().hash(&mut hasher);
    let hash = hasher.finish();
    Some((hash << 1) | 1)
}
