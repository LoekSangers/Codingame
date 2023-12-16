use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;

use super::traits::*;

pub const C: f64 = 1_f64;

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
    expanded: Cell<bool>,

    pub wins: Cell<f64>,
    pub visits: Cell<f64>,
    
    pub uct: Cell<f64>,
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
            uct: Cell::new(0.),
            expanded: Cell::new(false),
        }
    }

    pub fn create_child(state: S, parent: Weak<MctsNode<P, S, R, A>>) -> Self {
        MctsNode {
            unvisited_actions: RefCell::new(state.possible_actions()),
            state,
            parent,
            children: RefCell::new(HashMap::new()),
            wins: Cell::new(0.),
            visits: Cell::new(1.),
            uct: Cell::new(0.),
            expanded: Cell::new(false),
        }
    }

    pub fn best_child(&self) -> Rc<MctsNode<P, S, R, A>> {
        let children = self.children.borrow();
        if !children.is_empty() {
            let best = children
                .values()
                .reduce(|acc, node| {
                    eprintln!("{} / {} = {}", node.wins.get(), node.visits.get(), node.wins.get()/ node.visits.get());
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

    pub fn uct(&self, visits: f64) -> f64 {
        self.wins.get() / self.visits.get() - C * (self.visits.get() / visits)
    }

    pub fn select(self_ref: Rc<MctsNode<P, S, R, A>>) -> Rc<MctsNode<P, S, R, A>> {

        let uc = self_ref.unvisited_actions.borrow();
        let children = self_ref.children.borrow();
        let fully_expanded = self_ref.expanded.get();
        if fully_expanded && !children.is_empty(){       
            let selection = self_ref
                .children
                .borrow()
                .values()
                .reduce(|acc, node| {
                    if acc.uct.get() > node.uct.get() {
                        acc
                    } else {
                        node
                    }
                })
                .unwrap()
                .clone();
            MctsNode::select(selection)
        } else if !fully_expanded {//expand the node with one option
            drop(uc);
            drop(children);
            
            let mut unvisited_actions = self_ref.unvisited_actions.borrow_mut();
            let next_action = unvisited_actions.pop();
            drop(unvisited_actions);

            match next_action {
                Some(action) => {
                    let mut children = self_ref.children.borrow_mut();
                    let state = self_ref.state.perform_action_copy(&action);

                    let child = Rc::new(MctsNode::create_child(state, Rc::downgrade(&self_ref)));

                    children.insert(
                        action.clone(),
                        Rc::clone(&child),
                    );

                    child
                }
                None => {
                    self_ref.expanded.set(true);
                    MctsNode::select(self_ref)
                }
            }        
        } else {
            drop(uc);
            drop(children);
            self_ref
        }
    }

    pub fn backpropagate(&self, result: &R) {
        self.visits.set(self.visits.get() + 1_f64);
        self.wins
            .set(self.wins.get() + self.state.current_player().reward(result));
        if let Some(parent) = self.parent.upgrade() {
            parent.backpropagate(result);
            
            self.uct.set(self.uct(parent.visits.get() / parent.children.borrow().len() as f64))   
        }
        
    }
}
