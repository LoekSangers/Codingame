use std::cmp::Ordering;

use crate::masks::win_masks_for_move;

use super::{player::Player, game_state::BoardState};

#[derive(Clone)]
pub struct GlobalBoard {
    // LSB first; drawn if both x&y
    x: usize,
    o: usize,
    x_won: u16,
    o_won: u16,
    pub playable_boards: Vec<usize>,
}

impl Default for GlobalBoard {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalBoard {
    #[inline]
    pub fn new() -> GlobalBoard {
        GlobalBoard {
            x: 0,
            o: 0,
            x_won: 0,
            o_won: 0,
            playable_boards: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        }
    }

    #[inline]
    pub(in crate) fn in_play(&self, board: usize) -> bool {
        self.playable_boards.contains(&board)
    }

    #[inline]
    pub fn set(&mut self, board: usize, state: BoardState) -> BoardState {
        let bit = 1_usize << board;
        match state {
            BoardState::Drawn => {
                self.playable_boards.retain(|&x| x != board);

                self.x |= bit;
                self.o |= bit;

                if self.playable_boards.is_empty() {
                    match self.x_won.cmp(&self.o_won) {
                        Ordering::Greater => BoardState::Won(Player::X),
                        Ordering::Less => BoardState::Won(Player::O),
                        Ordering::Equal => BoardState::Drawn,
                    }
                } else {
                    BoardState::InPlay
                }
            }
            BoardState::InPlay => BoardState::InPlay,
            BoardState::Won(Player::X) => {
                self.playable_boards.retain(|&x| x != board);

                self.x_won += 1;
                self.x |= bit;

                if win_masks_for_move(bit)
                    .iter()
                    .any(|&win_mask| self.x & win_mask == win_mask)
                {
                    return BoardState::Won(Player::X);
                }

                if self.playable_boards.is_empty() {
                    match self.x_won.cmp(&self.o_won) {
                        Ordering::Greater => BoardState::Won(Player::X),
                        Ordering::Less => BoardState::Won(Player::O),
                        Ordering::Equal => BoardState::Drawn,
                    }
                } else {
                    BoardState::InPlay
                }
            }
            BoardState::Won(Player::O) => {
                self.playable_boards.retain(|&x| x != board);

                self.o_won += 1;
                self.o |= bit;

                if win_masks_for_move(bit)
                    .iter()
                    .any(|&win_mask| self.o & win_mask == win_mask)
                {
                    return BoardState::Won(Player::O);
                }

                if self.playable_boards.is_empty() {
                    match self.x_won.cmp(&self.o_won) {
                        Ordering::Greater => BoardState::Won(Player::X),
                        Ordering::Less => BoardState::Won(Player::O),
                        Ordering::Equal => BoardState::Drawn,
                    }
                } else {
                    BoardState::InPlay
                }
            }
        }
    }
}