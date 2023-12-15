use super::game::masks::local_to_global;
use super::game::masks::BOARD_MASK;
use std::hash::Hash;

use super::mcts::traits::GameAction;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Action {
    bits: usize,
}
impl Hash for Action {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {
        self.bits;
    }
}

impl GameAction for Action{}

impl Action {
    //#[inline]
    pub fn from_coords(global: usize, local: usize) -> Self {
        Self {
            bits: (global << 9 | local),
        }
    }
    //#[inline]
    pub fn global(self) -> usize {
        self.bits >> 9
    }
    //#[inline]
    pub fn local(self) -> usize {
        self.bits & BOARD_MASK
    }

    //#[inline]
    pub fn print(&self) {
        let row = self.global() / 3 * 3 + local_to_global(self.local()) / 3;
        let col = self.global() % 3 * 3 + local_to_global(self.local()) % 3;
        println!("{} {}", row, col);
    }
}