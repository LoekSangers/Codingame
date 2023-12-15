

use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;

use super::masks::local_to_global;

use super::mcts::traits::GameState;

use super::mcts::traits::GameResult;

use super::player::Player;
use super::local_board::LocalBoards;
use super::global_board::GlobalBoard;
use super::game_action::Action;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum UTTTResult {
    Won(Player),
    Drawn,
    InPlay,
}

impl GameResult for UTTTResult {}

#[derive(Clone)]
pub struct State {
    player: Player,
    pub last_action: Option<Action>,
    pub last_local_move: Option<usize>,
    pub local_boards: LocalBoards,
    pub global_states: GlobalBoard,
    pub result: UTTTResult,
}

impl Default for State {
    //#[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl GameState<Player, UTTTResult, Action> for State {
    fn current_player(&self) -> Player {
        self.player
    }

    fn next_player(&self) -> Player {
        self.player.other()
    }

    fn last_action(&self) -> Option<Action> {
       self.last_action
    }

    fn outcome(&self) -> UTTTResult {
        self.result
    }
    
    fn playable(&self) -> bool {
        self.result == UTTTResult::InPlay
    }

    //#[inline]
    fn possible_actions(&self) -> Vec<Action> {
        match self.last_local_move {
            Some(last_local) => {
                let global = local_to_global(last_local);
                if self.global_states.in_play(global) {
                    self.local_boards
                        .local_moves(global)
                        .iter()
                        .map(|&local| Action::from_coords(global, local))
                        .collect()
                } else {
                    self.global_states
                        .playable_boards
                        .iter()
                        .flat_map(|&global| {
                            self.local_boards
                                .local_moves(global)
                                .iter()
                                .map(move |&local| Action::from_coords(global, local))
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
                        .map(move |&local| Action::from_coords(global, local))
                })
                .collect(),
        }
    }

    fn perform_action_copy(&self, action: &Action) -> Self {
        let mut new_state = self.clone();        
        
        let board_state = new_state.local_boards.set(action.global(), action.local(), new_state.player);

        new_state.result = new_state.global_states.set(action.global(), board_state);
        new_state.player = new_state.player.other();
        new_state.last_action = Some(*action);
        new_state.last_local_move = Some(action.local());

        new_state
    }

    //#[inline]
    fn simulate_game(mut self, rng: &mut ThreadRng) -> UTTTResult {
        loop {
            let action = self.random_move(rng);

            match action {
                Some(action) => {
                    let board_state = self.local_boards.set(action.global(), action.local(), self.player);

                    self.result = self.global_states.set(action.global(), board_state);
                    self.player = self.player.other();
                    self.last_local_move = Some(action.local());
        
                    if self.result != UTTTResult::InPlay{
                        return self.result;
                    }
                },
                None => return UTTTResult::Drawn
            }

            
        }
    }
}

impl State {
    //#[inline]
    pub fn new() -> State {
        State {
            player: Player::X,
            last_action: None,
            last_local_move: None,
            local_boards: Default::default(),
            global_states: Default::default(),
            result: UTTTResult::InPlay,
        }
    }    

    //#[inline]
    pub fn random_move(&self, rng: &mut ThreadRng) -> Option<Action> {
        let global = local_to_global(self.last_local_move.unwrap());
        if self.global_states.in_play(global) {
            self.local_boards
                .local_moves(global)
                .iter()
                .choose(rng)
                .map(|&local| Action::from_coords(global, local))
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
                        .map(|&local| Action::from_coords(global, local))
                        .unwrap()
                })
        }
    }
}

