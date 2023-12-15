use crate::{game::masks, mcts:: Action, masks::local_to_global};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GameMove {
    bits: usize,
}

impl Action for GameMove{}

impl GameMove {
    #[inline]
    pub fn from_coords(global: usize, local: usize) -> Self {
        Self {
            bits: (global << 9 | local),
        }
    }
    #[inline]
    pub fn global(self) -> usize {
        self.bits >> 9
    }
    #[inline]
    pub fn local(self) -> usize {
        self.bits & masks::BOARD_MASK
    }

    #[inline]
    pub fn print(&self) {
        let row = self.global() / 3 * 3 + local_to_global(self.local()) / 3;
        let col = self.global() % 3 * 3 + local_to_global(self.local()) % 3;
        println!("{} {}", row, col);
    }
}