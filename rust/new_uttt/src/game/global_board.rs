use std::cmp::Ordering;

use super::masks::win_masks_for_move;

use super::player::Player;
use super::game_state::UTTTResult;

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
    //#[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalBoard {
    //#[inline]
    pub fn new() -> GlobalBoard {
        GlobalBoard {
            x: 0,
            o: 0,
            x_won: 0,
            o_won: 0,
            playable_boards: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        }
    }

    //#[inline]
    pub fn in_play(&self, board: usize) -> bool {
        self.playable_boards.contains(&board)
    }

    //#[inline]
    pub fn set(&mut self, board: usize, state: UTTTResult) -> UTTTResult {
        let bit = 1_usize << board;
        match state {
            UTTTResult::Drawn => {
                match self.playable_boards.iter().position(|&loc| loc == board) {
                    Some(index) => self.playable_boards.swap_remove(index),
                    None => 0
                }; 

                self.x |= bit;
                self.o |= bit;

                if self.playable_boards.is_empty() {
                    match self.x_won.cmp(&self.o_won) {
                        Ordering::Greater => UTTTResult::Won(Player::X),
                        Ordering::Less => UTTTResult::Won(Player::O),
                        Ordering::Equal => UTTTResult::Drawn,
                    }
                } else {
                    UTTTResult::InPlay
                }
            }
            UTTTResult::InPlay => UTTTResult::InPlay,
            UTTTResult::Won(Player::X) => {
                match self.playable_boards.iter().position(|&loc| loc == board) {
                    Some(index) => self.playable_boards.swap_remove(index),
                    None => 0
                }; 

                self.x_won += 1;
                self.x |= bit;

                if win_masks_for_move(bit)
                    .iter()
                    .any(|&win_mask| self.x & win_mask == win_mask)
                {
                    return UTTTResult::Won(Player::X);
                }

                if self.playable_boards.is_empty() {
                    match self.x_won.cmp(&self.o_won) {
                        Ordering::Greater => UTTTResult::Won(Player::X),
                        Ordering::Less => UTTTResult::Won(Player::O),
                        Ordering::Equal => UTTTResult::Drawn,
                    }
                } else {
                    UTTTResult::InPlay
                }
            }
            UTTTResult::Won(Player::O) => {
                match self.playable_boards.iter().position(|&loc| loc == board) {
                    Some(index) => self.playable_boards.swap_remove(index),
                    None => 0
                }; 

                self.o_won += 1;
                self.o |= bit;

                if win_masks_for_move(bit)
                    .iter()
                    .any(|&win_mask| self.o & win_mask == win_mask)
                {
                    return UTTTResult::Won(Player::O);
                }

                if self.playable_boards.is_empty() {
                    match self.x_won.cmp(&self.o_won) {
                        Ordering::Greater => UTTTResult::Won(Player::X),
                        Ordering::Less => UTTTResult::Won(Player::O),
                        Ordering::Equal => UTTTResult::Drawn,
                    }
                } else {
                    UTTTResult::InPlay
                }
            }
        }
    }
}