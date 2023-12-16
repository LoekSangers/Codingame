use std::cell::RefCell;
use std::rc::Rc;
use std::time::{self, Instant};

use super::cg_rand::Rng;
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
        MctsTree {
            root: RefCell::new(node_ref),
        }
    }

    pub fn best_child(&self) -> Rc<MctsNode<P, S, R, A>>{
        self.root.borrow().best_child()
    }

    pub fn expand_tree(&self, begin: Instant, duration: time::Duration, rng: &mut Box<Rng>, depth: usize) {
        let mut count = 0_u32;

        let root_ref = Rc::clone(&self.root.borrow());
        while begin.elapsed() < duration {
            let selected = MctsNode::select(Rc::clone(&root_ref), depth);
            let end_state = selected.state.clone();
            if end_state.playable() {
                let result = end_state.simulate_game(rng);
                selected.backpropagate(&result);
            } else {
                selected.backpropagate(&end_state.outcome());
            }
            count += 1;
        }
        eprintln!("{}", count);
    }

    pub fn move_down(&self, action: A) {
        let child: Rc<MctsNode<P, S, R, A>> = Rc::clone(&self.root.borrow().children.borrow().get(&action).unwrap());
        self.root.replace(child);
    }
}
