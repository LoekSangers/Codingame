use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;

use super::traits::*;

pub const C: f64 = 1.414_f64;

pub struct MctsNode<P, S, R, A>
where
    P: GamePlayer<R>,
    S: GameState<P, R, A>,
    R: GameResult,
    A: GameAction,
{
    pub state: S,

    parent: Weak<MctsNode<P, S, R, A>>,
    pub children: RefCell<HashMap<A, Rc<MctsNode<P, S, R, A>>>>,

    unvisited_actions: RefCell<Vec<A>>,

    wins: Cell<f64>,
    visits: Cell<f64>,
}

impl<P, S, R, A> MctsNode<P, S, R, A>
where
    P: GamePlayer<R>,
    S: GameState<P, R, A>,
    R: GameResult,
    A: GameAction,
{
    pub fn new(state: S) -> Self {
        MctsNode {
            unvisited_actions: RefCell::new(state.possible_actions()),
            state,
            parent: Weak::new(),
            children: RefCell::new(HashMap::new()),
            wins: Cell::new(0.),
            visits: Cell::new(1.),
        }
    }

    pub fn create_child(state: S, parent: Weak<MctsNode<P, S, R, A>>) -> Self {
        MctsNode {
            unvisited_actions: RefCell::new(state.possible_actions()),
            state,
            parent,
            children: RefCell::new(HashMap::new()),
            wins: Cell::new(0_f64),
            visits: Cell::new(1_f64),
        }
    }

    pub fn uct(&self, visits: f64) -> f64 {
        self.wins.get() / self.visits.get() + C * (visits / self.visits.get())
    }

    pub fn best_child(&self) -> Rc<MctsNode<P, S, R, A>> {
        let children = self.children.borrow();
        if !children.is_empty() {
            let best = children
                .values()
                .reduce(|acc, node| {
                    if acc.wins.get() / acc.visits.get() > node.wins.get() / node.visits.get() {
                        acc
                    } else {
                        node
                    }
                })
                .unwrap()
                .clone();
            best
        } else {
            panic!("There is no best move")
        }
    }

    pub fn select(self_ref: Rc<MctsNode<P, S, R, A>>) -> Rc<MctsNode<P, S, R, A>> {
        let uc = self_ref.unvisited_actions.borrow();
        let children = self_ref.children.borrow();
        if uc.is_empty() && !children.is_empty() {
            let visits = match self_ref.parent.upgrade() {
                Some(parent_node) => parent_node.visits.get().ln(),
                None => self_ref.visits.get().ln(),
            };
            self_ref
                .children
                .borrow()
                .values()
                .reduce(|acc, node| {
                    if acc.uct(visits) > node.uct(visits) {
                        acc
                    } else {
                        node
                    }
                })
                .unwrap()
                .clone()
        } else if children.is_empty() {
            drop(uc);
            drop(children);
            Self::expand(Rc::clone(&self_ref));
            let children = self_ref.children.borrow();

            children.values().next().unwrap().clone()
        } else {
            drop(uc);
            drop(children);
            self_ref
        }
    }

    pub fn expand(parent_ref: Rc<MctsNode<P, S, R, A>>) {
        let uc = parent_ref.unvisited_actions.borrow();
        let mut children = parent_ref.children.borrow_mut();
        if !uc.is_empty() {
            uc.iter().for_each(|a| {
                let state = parent_ref.state.perform_action_copy(&a);

                children.insert(
                    a.clone(),
                    Rc::new(MctsNode::create_child(state, Rc::downgrade(&parent_ref))),
                );
            });
        }
        drop(uc);

        parent_ref.unvisited_actions.borrow_mut().clear();
    }

    pub fn backpropagate(&self, result: &R) {
        self.visits.set(self.visits.get() + 1_f64);
        self.wins
            .set(self.wins.get() + self.state.next_player().reward(result));
        match &self.parent.upgrade() {
            Some(parent_node) => parent_node.backpropagate(result),
            None => (),
        }
    }
}
