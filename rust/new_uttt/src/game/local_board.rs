use super::masks::LOCAL_MOVES;
use super::masks::win_masks_for_move;
use super::masks::BOARD_MASK;

use super::game_state::UTTTResult;
use super::player::Player;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalBoards {
    boards_x: [usize; 9],
    boards_o: [usize; 9],
    legal_moves: [Vec<usize>; 9],
}

impl Default for LocalBoards {
    fn default() -> Self {
        Self {
            boards_x: Default::default(),
            boards_o: Default::default(),
            legal_moves: [
                Vec::from(LOCAL_MOVES),
                Vec::from(LOCAL_MOVES),
                Vec::from(LOCAL_MOVES),
                Vec::from(LOCAL_MOVES),
                Vec::from(LOCAL_MOVES),
                Vec::from(LOCAL_MOVES),
                Vec::from(LOCAL_MOVES),
                Vec::from(LOCAL_MOVES),
                Vec::from(LOCAL_MOVES),
            ],
        }
    }
}

impl LocalBoards {
    //#[inline]
    pub fn local_moves(&self, global: usize) -> &Vec<usize> {
        &self.legal_moves[global]
    }

    //#[inline]
    pub fn set(&mut self, global: usize, local: usize, player: Player) -> UTTTResult {
        match self.legal_moves[global].iter().position(|&loc| loc == local) {
            Some(index) => self.legal_moves[global].swap_remove(index),
            None => 0
        };        
        match player {
            Player::O => {
                self.boards_o[global] |= local;
                if win_masks_for_move(local)
                    .iter()
                    .any(|&win_mask| self.boards_o[global] & win_mask == win_mask)
                {
                    return UTTTResult::Won(Player::O);
                }
            }
            Player::X => {
                self.boards_x[global] |= local;
                if win_masks_for_move(local)
                    .iter()
                    .any(|&win_mask| self.boards_x[global] & win_mask == win_mask)
                {
                    return UTTTResult::Won(Player::X);
                }
            }
        }
        if (self.boards_o[global] | self.boards_x[global]) == BOARD_MASK {
            UTTTResult::Drawn
        } else {
            UTTTResult::InPlay
        }
    }
}