use rand::rngs::ThreadRng;

pub trait GameResult {}

pub trait GameAction: Eq + Clone + std::hash::Hash {}

pub trait GamePlayer<R: GameResult>{
    fn reward(&self, result: &R) -> f64;
}
pub trait GameState<P, R, A>: Clone
where
    P: GamePlayer<R>,
    R: GameResult,
    A: GameAction,
{
    fn current_player(&self) -> P;
    fn next_player(&self) -> P;

    fn last_action(&self) -> Option<A>;

    fn possible_actions(&self) -> Vec<A>;

    fn perform_action_copy(&self, action: &A) -> Self;

    fn simulate_game(self, rng: &mut ThreadRng) -> R;

    fn outcome(&self) -> R;
    fn playable(&self) -> bool;
}
