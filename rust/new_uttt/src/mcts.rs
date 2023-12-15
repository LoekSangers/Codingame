use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::time;

use rand::rngs::ThreadRng;

use crate::{GameState, BoardState};

pub const C: f64 = 0.6_f64;

pub const WIN_SCORE: f64 = 1_f64;
pub const DRAW_SCORE: f64 = 0.5_f64;

pub enum GameEnd {
    Win,
    Loss,
    Draw,
}

impl GameEnd {
    pub fn opposite(&self) -> Self {
        match &self {
            GameEnd::Draw => GameEnd::Draw,
            GameEnd::Win => GameEnd::Loss,
            GameEnd::Loss => GameEnd::Win,
        }
    }
}

pub trait Action: Eq + Clone + Copy {}

pub struct Node<A>
where
    A: Action,
{
    pub last_action: Option<A>,
    parent: Weak<Node<A>>,
    pub children: RefCell<Vec<Rc<Node<A>>>>,
    pub unvisited_moves: RefCell<Vec<A>>,
    wins: Cell<f64>,
    visits: Cell<f64>,
}

impl<A> Node<A>
where
    A: Action,
{
    pub fn new() -> Self {
        Node {
            last_action: None,
            parent: Weak::new(),
            children: RefCell::new(vec![]),
            unvisited_moves: RefCell::new(vec![]),
            wins: Cell::new(0_f64),
            visits: Cell::new(0_f64),
        }
    }
    pub fn create_child(action: A, parent: Weak<Node<A>>) -> Self {
        Node {
            last_action: Some(action),
            parent: parent,
            children: RefCell::new(vec![]),
            unvisited_moves: RefCell::new(vec![]),
            wins: Cell::new(0_f64),
            visits: Cell::new(0_f64),
        }
    }

    pub fn mcts(self_ref: Rc<Node<A>>, game: &mut GameState, run_time_nano: u32, rng: &mut ThreadRng){
        let begin = time::Instant::now();
        let mut count = 0_u32;
        let duration = time::Duration::new(0, run_time_nano);

        while begin.elapsed() < duration {
            let selected = Self::select(Rc::clone(&self_ref));
            selected.simulate(game, rng);
            count += 1;
        }
        eprintln!("{}", count);
    }

    pub fn uct(&self) -> f64 {
        match &self.parent.upgrade() {
            Some(parent_node) => {
                self.wins.get() / self.visits.get()
                    + C * (parent_node.visits.get().ln() / self.visits.get()).sqrt()
            }
            None => {
                self.wins.get() / self.visits.get()
                    + C * (self.visits.get().ln() / self.visits.get()).sqrt()
            }
        }
    }

    pub fn best_child(&self) -> Rc<Node<A>> {
        let children = self.children.borrow();
        if !children.is_empty() {
            children
                .iter()
                .reduce(|acc, node| {
                    if acc.wins.get() / acc.visits.get() > node.wins.get() / node.visits.get() {
                        acc
                    } else {
                        node
                    }
                })
                .unwrap().clone()
        } else {
            panic!("There is no best move")
        }
    }

    pub fn select(self_ref: Rc<Node<A>>) -> Rc<Node<A>> {
        let uc = self_ref.unvisited_moves.borrow();
        let children = self_ref.children.borrow();
        if uc.is_empty() && !children.is_empty() {
            self_ref
                .children
                .borrow()
                .iter()
                .reduce(|acc, node| if acc.uct() > node.uct() { acc } else { node })
                .unwrap()
                .clone()
        } else if children.is_empty(){
            drop(uc);
            drop(children);
            Self::expand(Rc::clone(&self_ref));
            let children = self_ref.children.borrow();

            children.first().unwrap().clone()
        } else {
            drop(uc);
            drop(children);
            self_ref
        }
    }

    pub fn expand(parent_ref: Rc<Node<A>>) {

        let uc = parent_ref.unvisited_moves.borrow();
        let mut children = parent_ref.children.borrow_mut();
        if !uc.is_empty() {
            uc.iter().for_each(|&m| {
                children.push(Rc::new(Node::create_child(m, Rc::downgrade(&parent_ref))))
            });
        }
        drop(uc);

        parent_ref.unvisited_moves.borrow_mut().clear();
    }

    pub fn simulate(&self, game: &mut GameState, rng: &mut ThreadRng) {
        while game.game_state == BoardState::InPlay {
            game.inplace_move(&game.random_move(rng));
        }
    }

    pub fn backpropagate(&self, end: GameEnd) {
        self.visits.set(self.visits.get() + 1_f64);
        match end {
            GameEnd::Win => self.wins.set(self.wins.get() + WIN_SCORE),
            GameEnd::Draw => self.wins.set(self.wins.get() + DRAW_SCORE),
            GameEnd::Loss => (),
        };
        match &self.parent.upgrade() {
            Some(parent_node) => parent_node.backpropagate(end.opposite()),
            None => (),
        }
    }
}
