use crate::masks::{LOCAL_MOVES, win_masks_for_move, BOARD_MASK};

use super::{game_state::BoardState, player::Player};

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
    #[inline]
    pub fn local_moves(&self, global: usize) -> &Vec<usize> {
        &self.legal_moves[global]
    }

    #[inline]
    pub fn set(&mut self, global: usize, local: usize, player: Player) -> BoardState {
        self.legal_moves[global].retain(|&loc| loc != local);
        match player {
            Player::O => {
                self.boards_o[global] |= local;
                if win_masks_for_move(local)
                    .iter()
                    .any(|&win_mask| self.boards_o[global] & win_mask == win_mask)
                {
                    return BoardState::Won(Player::O);
                }
            }
            Player::X => {
                self.boards_x[global] |= local;
                if win_masks_for_move(local)
                    .iter()
                    .any(|&win_mask| self.boards_x[global] & win_mask == win_mask)
                {
                    return BoardState::Won(Player::X);
                }
            }
        }
        if (self.boards_o[global] | self.boards_x[global]) == BOARD_MASK {
            BoardState::Drawn
        } else {
            BoardState::InPlay
        }
    }
}