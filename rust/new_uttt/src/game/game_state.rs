

use rand::{rngs::ThreadRng, seq::IteratorRandom};

use crate::{GameMove, masks::local_to_global};

use super::{player::Player, local_board::LocalBoards, global_board::GlobalBoard};

#[derive(PartialEq, Clone, Copy)]
pub enum BoardState {
    Won(Player),
    Drawn,
    InPlay,
}

#[derive(Clone)]
pub struct GameState {
    pub player: Player,
    pub last_local_move: Option<usize>,
    pub local_boards: LocalBoards,
    pub global_states: GlobalBoard,
    pub game_state: BoardState,
}

impl Default for GameState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    #[inline]
    pub fn new() -> GameState {
        GameState {
            player: Player::X,
            last_local_move: None,
            local_boards: Default::default(),
            global_states: Default::default(),
            game_state: BoardState::InPlay,
        }
    }

    #[inline]
    pub fn inplace_move(&mut self, m: &GameMove) -> BoardState {
        let board_state = self.local_boards.set(m.global(), m.local(), self.player);

        self.game_state = self.global_states.set(m.global(), board_state);
        self.player = self.player.other();
        self.last_local_move = Some(m.local());

        board_state
    }

    #[inline]
    pub fn all_moves(&self) -> Vec<GameMove> {
        match self.last_local_move {
            Some(last_local) => {
                let global = local_to_global(last_local);
                if self.global_states.in_play(global) {
                    self.local_boards
                        .local_moves(global)
                        .iter()
                        .map(|&local| GameMove::from_coords(global, local))
                        .collect()
                } else {
                    self.global_states
                        .playable_boards
                        .iter()
                        .flat_map(|&global| {
                            self.local_boards
                                .local_moves(global)
                                .iter()
                                .map(move |&local| GameMove::from_coords(global, local))
                        })
                        .collect()
                }
            }
            _ => self
                .global_states
                .playable_boards
                .iter()
                .flat_map(|&global| {
                    self.local_boards
                        .local_moves(global)
                        .iter()
                        .map(move |&local| GameMove::from_coords(global, local))
                })
                .collect(),
        }
    }

    #[inline]
    pub fn random_move(&self, rng: &mut ThreadRng) -> GameMove {
        let global = local_to_global(self.last_local_move.unwrap());
        if self.global_states.in_play(global) {
            self.local_boards
                .local_moves(global)
                .iter()
                .choose(rng)
                .map(|&local| GameMove::from_coords(global, local))
                .unwrap()
        } else {
            self.global_states
                .playable_boards
                .iter()
                .choose(rng)
                .map(|&global| {
                    self.local_boards
                        .local_moves(global)
                        .iter()
                        .choose(rng)
                        .map(|&local| GameMove::from_coords(global, local))
                        .unwrap()
                })
                .unwrap()
        }
    }

    #[inline]
    pub fn random_playout(&mut self, rng: &mut ThreadRng) {
        while self.game_state == BoardState::InPlay {
            self.inplace_move(&self.random_move(rng));
        }
    }

    #[inline]
    pub fn player(&self) -> Player {
        self.player
    }
}

