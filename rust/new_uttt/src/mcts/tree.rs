use std::cell::RefCell;
use std::rc::Rc;
use std::time;

use rand::rngs::ThreadRng;

use super::traits::*;

use super::node::MctsNode;

pub struct MctsTree<P, S, R, A>
where
    P: GamePlayer<R>,
    S: GameState<P, R, A>,
    R: GameResult,
    A: GameAction,
{
    pub root: RefCell<Rc<MctsNode<P, S, R, A>>>,
}

impl<P, S, R, A> MctsTree<P, S, R, A>
where
    P: GamePlayer<R>,
    S: GameState<P, R, A>,
    R: GameResult,
    A: GameAction,
{
    pub fn new(state: S) -> Self {
        let node_ref = Rc::new(MctsNode::new(state));
        MctsNode::select(Rc::clone(&node_ref));
        MctsTree {
            root: RefCell::new(node_ref),
        }
    }

    pub fn expand_tree(&self, run_time_nano: u32, rng: &mut ThreadRng) {
        let begin = time::Instant::now();
        let mut count = 0_u32;
        let duration = time::Duration::new(0, run_time_nano);

        let root_ref = Rc::clone(&self.root.borrow());
        while begin.elapsed() < duration {
            let selected = MctsNode::select(Rc::clone(&root_ref));
            let end_state = selected.state.clone();
            if end_state.playable() {
                let result = end_state.simulate_game(rng);
                MctsNode::backpropagate(Rc::clone(&selected), &result);
            } else {
                MctsNode::backpropagate(Rc::clone(&selected), &end_state.outcome());
            }
            count += 1;
        }
        eprintln!("{}", count);
    }

    pub fn move_down(&self, action: A) {
        MctsNode::expand(Rc::clone(&self.root.borrow()));

        let child: Rc<MctsNode<P, S, R, A>> =
            Rc::clone(self.root.borrow().children.borrow().get(&action).unwrap());

        self.root.replace(child);
    }
}
