use super::game_state::UTTTResult;
use super::mcts::traits::GamePlayer;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    X,
    O,
}

impl Player {
    //#[inline]
    pub const fn other(&self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

impl GamePlayer<UTTTResult> for Player{
    fn reward(&self, result: &UTTTResult) -> f64 {
        match result{
            UTTTResult::Won(winner) => {
                if self == winner {
                    1.
                }else{
                    0.
                }
            },
            UTTTResult::Drawn => 0.3,
            UTTTResult::InPlay => 0.,
        }
    }
}