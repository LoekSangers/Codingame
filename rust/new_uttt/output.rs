pub mod game {
    pub mod masks {
        pub const fn win_masks_for_move(local: usize) -> &'static [usize] {
            match local {
                0b000_000_001 => &[0b000_000_111, 0b001_001_001, 0b100_010_001],
                0b000_000_010 => &[0b000_000_111, 0b010_010_010],
                0b000_000_100 => &[0b000_000_111, 0b100_100_100, 0b001_010_100],
                0b000_001_000 => &[0b000_111_000, 0b001_001_001],
                0b000_010_000 => &[0b000_111_000, 0b010_010_010, 0b001_010_100, 0b100_010_001],
                0b000_100_000 => &[0b000_111_000, 0b100_100_100],
                0b001_000_000 => &[0b111_000_000, 0b001_001_001, 0b001_010_100],
                0b010_000_000 => &[0b111_000_000, 0b010_010_010],
                0b100_000_000 => &[0b111_000_000, 0b100_100_100, 0b100_010_001],
                _ => &[],
            }
        }
        pub const LOCAL_MOVES: &[usize] = &[
            0b000_000_001,
            0b000_000_010,
            0b000_000_100,
            0b000_001_000,
            0b000_010_000,
            0b000_100_000,
            0b001_000_000,
            0b010_000_000,
            0b100_000_000,
        ];
        pub const fn local_to_global(local: usize) -> usize {
            match local {
                0b000_000_001 => 0,
                0b000_000_010 => 1,
                0b000_000_100 => 2,
                0b000_001_000 => 3,
                0b000_010_000 => 4,
                0b000_100_000 => 5,
                0b001_000_000 => 6,
                0b010_000_000 => 7,
                0b100_000_000 => 8,
                _ => 16,
            }
        }
        pub const BOARD_MASK: usize = 0b111_111_111;
    }
    pub mod game_action {
        use super::game::masks::local_to_global;
        use super::game::masks::BOARD_MASK;
        use std::hash::Hash;
        use super::mcts::traits::GameAction;
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct Action {
            bits: usize,
        }
        impl Hash for Action {
            fn hash<H: std::hash::Hasher>(&self, _: &mut H) {
                self.bits;
            }
        }
        impl GameAction for Action {}
        impl Action {
            pub fn from_coords(global: usize, local: usize) -> Self {
                Self { bits: (global << 9 | local) }
            }
            pub fn global(self) -> usize {
                self.bits >> 9
            }
            pub fn local(self) -> usize {
                self.bits & BOARD_MASK
            }
            pub fn print(&self) {
                let row = self.global() / 3 * 3 + local_to_global(self.local()) / 3;
                let col = self.global() % 3 * 3 + local_to_global(self.local()) % 3;
                println!("{} {}", row, col);
            }
        }
    }
    pub mod game_state {
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
                                    self.local_boards.local_moves(global).iter().map(
                                        move |&local| {
                                            Action::from_coords(global, local)
                                        },
                                    )
                                })
                                .collect()
                        }
                    }
                    _ => {
                        self.global_states
                            .playable_boards
                            .iter()
                            .flat_map(|&global| {
                                self.local_boards.local_moves(global).iter().map(
                                    move |&local| {
                                        Action::from_coords(global, local)
                                    },
                                )
                            })
                            .collect()
                    }
                }
            }
            fn perform_action_copy(&self, action: &Action) -> Self {
                let mut new_state = self.clone();
                let board_state = new_state.local_boards.set(
                    action.global(),
                    action.local(),
                    new_state.player,
                );
                new_state.result = new_state.global_states.set(action.global(), board_state);
                new_state.player = new_state.player.other();
                new_state.last_action = Some(*action);
                new_state.last_local_move = Some(action.local());
                new_state
            }
            fn simulate_game(mut self, rng: &mut ThreadRng) -> UTTTResult {
                loop {
                    let action = self.random_move(rng);
                    match action {
                        Some(action) => {
                            let board_state = self.local_boards.set(
                                action.global(),
                                action.local(),
                                self.player,
                            );
                            self.result = self.global_states.set(action.global(), board_state);
                            self.player = self.player.other();
                            self.last_local_move = Some(action.local());
                            if self.result != UTTTResult::InPlay {
                                return self.result;
                            }
                        }
                        None => return UTTTResult::Drawn,
                    }
                }
            }
        }
        impl State {
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
            pub fn random_move(&self, rng: &mut ThreadRng) -> Option<Action> {
                let global = local_to_global(self.last_local_move.unwrap());
                if self.global_states.in_play(global) {
                    self.local_boards
                        .local_moves(global)
                        .iter()
                        .choose(rng)
                        .map(|&local| Action::from_coords(global, local))
                } else {
                    self.global_states.playable_boards.iter().choose(rng).map(
                        |&global| {
                            self.local_boards
                                .local_moves(global)
                                .iter()
                                .choose(rng)
                                .map(|&local| Action::from_coords(global, local))
                                .unwrap()
                        },
                    )
                }
            }
        }
    }
    pub mod player {
        use super::game_state::UTTTResult;
        use super::mcts::traits::GamePlayer;
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Player {
            X,
            O,
        }
        impl Player {
            pub const fn other(&self) -> Player {
                match self {
                    Player::X => Player::O,
                    Player::O => Player::X,
                }
            }
        }
        impl GamePlayer<UTTTResult> for Player {
            fn reward(&self, result: &UTTTResult) -> f64 {
                match result {
                    UTTTResult::Won(winner) => if self == winner { 1. } else { 0. },
                    UTTTResult::Drawn => 0.3,
                    UTTTResult::InPlay => 0.,
                }
            }
        }
    }
    pub mod local_board {
        use super::masks::LOCAL_MOVES;
        use super::masks::win_masks_for_move;
        use super::masks::BOARD_MASK;
        use super::game_state::UTTTResult;
        use super::player::Player;
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
            pub fn local_moves(&self, global: usize) -> &Vec<usize> {
                &self.legal_moves[global]
            }
            pub fn set(&mut self, global: usize, local: usize, player: Player) -> UTTTResult {
                match self.legal_moves[global].iter().position(
                    |&loc| loc == local,
                ) {
                    Some(index) => self.legal_moves[global].swap_remove(index),
                    None => 0,
                };
                match player {
                    Player::O => {
                        self.boards_o[global] |= local;
                        if win_masks_for_move(local).iter().any(|&win_mask| {
                            self.boards_o[global] & win_mask == win_mask
                        })
                        {
                            return UTTTResult::Won(Player::O);
                        }
                    }
                    Player::X => {
                        self.boards_x[global] |= local;
                        if win_masks_for_move(local).iter().any(|&win_mask| {
                            self.boards_x[global] & win_mask == win_mask
                        })
                        {
                            return UTTTResult::Won(Player::X);
                        }
                    }
                }
                if (self.boards_o[global] | self.boards_x[global]) == BOARD_MASK {
                    UTTTResult::Drawn
                } else {
                    UTTTResult::InPlay
                }
            }
        }
    }
    pub mod global_board {
        use std::cmp::Ordering;
        use super::masks::win_masks_for_move;
        use super::player::Player;
        use super::game_state::UTTTResult;
        #[derive(Clone)]
        pub struct GlobalBoard {
            x: usize,
            o: usize,
            x_won: u16,
            o_won: u16,
            pub playable_boards: Vec<usize>,
        }
        impl Default for GlobalBoard {
            fn default() -> Self {
                Self::new()
            }
        }
        impl GlobalBoard {
            pub fn new() -> GlobalBoard {
                GlobalBoard {
                    x: 0,
                    o: 0,
                    x_won: 0,
                    o_won: 0,
                    playable_boards: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
                }
            }
            pub fn in_play(&self, board: usize) -> bool {
                self.playable_boards.contains(&board)
            }
            pub fn set(&mut self, board: usize, state: UTTTResult) -> UTTTResult {
                let bit = 1_usize << board;
                match state {
                    UTTTResult::Drawn => {
                        match self.playable_boards.iter().position(|&loc| loc == board) {
                            Some(index) => self.playable_boards.swap_remove(index),
                            None => 0,
                        };
                        self.x |= bit;
                        self.o |= bit;
                        if self.playable_boards.is_empty() {
                            match self.x_won.cmp(&self.o_won) {
                                Ordering::Greater => UTTTResult::Won(Player::X),
                                Ordering::Less => UTTTResult::Won(Player::O),
                                Ordering::Equal => UTTTResult::Drawn,
                            }
                        } else {
                            UTTTResult::InPlay
                        }
                    }
                    UTTTResult::InPlay => UTTTResult::InPlay,
                    UTTTResult::Won(Player::X) => {
                        match self.playable_boards.iter().position(|&loc| loc == board) {
                            Some(index) => self.playable_boards.swap_remove(index),
                            None => 0,
                        };
                        self.x_won += 1;
                        self.x |= bit;
                        if win_masks_for_move(bit).iter().any(|&win_mask| {
                            self.x & win_mask == win_mask
                        })
                        {
                            return UTTTResult::Won(Player::X);
                        }
                        if self.playable_boards.is_empty() {
                            match self.x_won.cmp(&self.o_won) {
                                Ordering::Greater => UTTTResult::Won(Player::X),
                                Ordering::Less => UTTTResult::Won(Player::O),
                                Ordering::Equal => UTTTResult::Drawn,
                            }
                        } else {
                            UTTTResult::InPlay
                        }
                    }
                    UTTTResult::Won(Player::O) => {
                        match self.playable_boards.iter().position(|&loc| loc == board) {
                            Some(index) => self.playable_boards.swap_remove(index),
                            None => 0,
                        };
                        self.o_won += 1;
                        self.o |= bit;
                        if win_masks_for_move(bit).iter().any(|&win_mask| {
                            self.o & win_mask == win_mask
                        })
                        {
                            return UTTTResult::Won(Player::O);
                        }
                        if self.playable_boards.is_empty() {
                            match self.x_won.cmp(&self.o_won) {
                                Ordering::Greater => UTTTResult::Won(Player::X),
                                Ordering::Less => UTTTResult::Won(Player::O),
                                Ordering::Equal => UTTTResult::Drawn,
                            }
                        } else {
                            UTTTResult::InPlay
                        }
                    }
                }
            }
        }
    }
    pub use super::*;
}
pub mod mcts {
    pub mod node {
        use std::cell::Cell;
        use std::cell::RefCell;
        use std::collections::HashMap;
        use std::rc::Rc;
        use std::rc::Weak;
        use std::thread::panicking;
        use super::traits::*;
        pub const C: f64 = 0.6_f64;
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
            exploration_score: Cell<f64>,
            max_child_exploration_score: Cell<f64>,
            child_to_explore: Cell<Weak<MctsNode<P, S, R, A>>>,
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
                    exploration_score: Cell::new(0.),
                    max_child_exploration_score: Cell::new(0.),
                    child_to_explore: Cell::new(Weak::new()),
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
                    exploration_score: Cell::new(0.),
                    max_child_exploration_score: Cell::new(0.),
                    child_to_explore: Cell::new(Weak::new()),
                }
            }
            pub fn uct(&self) -> f64 {
                match &self.parent.upgrade() {
                    Some(parent_node) => {
                        self.wins.get() / self.visits.get() +
                            C * (parent_node.visits.get().ln() / self.visits.get()).sqrt()
                    }
                    None => {
                        self.wins.get() / self.visits.get() +
                            C * (self.visits.get().ln() / self.visits.get()).sqrt()
                    }
                }
            }
            pub fn best_child(&self) -> Rc<MctsNode<P, S, R, A>> {
                let children = self.children.borrow();
                if !children.is_empty() {
                    let best = children
                        .values()
                        .reduce(|acc, node| if acc.wins.get() / acc.visits.get() >
                            node.wins.get() / node.visits.get()
                        {
                            acc
                        } else {
                            node
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
                    let child_ref = self_ref.child_to_explore.take().upgrade().unwrap();
                    self_ref.child_to_explore.set(Rc::downgrade(&child_ref));
                    child_ref
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
            pub fn backpropagate(self_ref: Rc<MctsNode<P, S, R, A>>, result: &R) {
                self_ref.visits.set(self_ref.visits.get() + 1_f64);
                self_ref.wins.set(
                    self_ref.wins.get() + self_ref.state.next_player().reward(result),
                );
                match &self_ref.parent.upgrade() {
                    Some(parent_node) => Self::backpropagate(Rc::clone(parent_node), result),
                    None => (),
                }
                let utc = self_ref.uct();
                self_ref.exploration_score.set(utc);
                match &self_ref.parent.upgrade() {
                    Some(parent_node) => {
                        if self_ref.exploration_score > parent_node.max_child_exploration_score {
                            parent_node.child_to_explore.set(Rc::downgrade(&self_ref));
                            parent_node.max_child_exploration_score.set(utc);
                        }
                    }
                    None => (),
                }
            }
        }
    }
    pub mod tree {
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
                MctsTree { root: RefCell::new(node_ref) }
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
    }
    pub mod traits {
        use rand::rngs::ThreadRng;
        pub trait GameResult {}
        pub trait GameAction: Eq + Clone + std::hash::Hash {}
        pub trait GamePlayer<R: GameResult> {
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
    }
}
extern crate rand ;
use std::io;
use game::game_action::Action;
use game::game_state::State;
use game::masks::LOCAL_MOVES;
use mcts::tree::MctsTree;
fn main() {
    codingame();
}
fn codingame() {
    let mut rng = rand::thread_rng();
    let inputs = read_input();
    let game = State::default();
    let mcts = MctsTree::new(game);
    let mut action: Action;
    if inputs.0 < 0 {
        action = Action::from_coords(4, LOCAL_MOVES[4]);
        mcts.move_down(action);
        mcts.expand_tree(999999995, &mut rng);
        println!("4 4");
    } else {
        let opponent_row = inputs.0;
        let opponent_col = inputs.1;
        action = Action::from_coords(
            (opponent_col / 3 + (opponent_row / 3) * 3) as usize,
            LOCAL_MOVES[(opponent_col % 3 + (opponent_row % 3) * 3) as usize],
        );
        mcts.move_down(action);
        mcts.expand_tree(999999995, &mut rng);
        let child = mcts.root.borrow().best_child();
        action = child.state.last_action.unwrap();
        mcts.root.replace(child);
        action.print();
    }
    loop {
        let inputs = read_input();
        let opponent_row = inputs.0;
        let opponent_col = inputs.1;
        let opp_move = Action::from_coords(
            (opponent_col / 3 + (opponent_row / 3) * 3) as usize,
            LOCAL_MOVES[(opponent_col % 3 + (opponent_row % 3) * 3) as usize],
        );
        mcts.move_down(opp_move);
        mcts.expand_tree(99999995, &mut rng);
        let child = mcts.root.borrow().best_child();
        action = child.state.last_action.unwrap();
        mcts.root.replace(child);
        action.print();
    }
}
fn read_input() -> (i8, i8) {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split_ascii_whitespace().collect::<Vec<_>>();
    let mut input_line_2 = String::new();
    io::stdin().read_line(&mut input_line_2).unwrap();
    let valid_action_count = input_line_2.trim().parse::<i32>().unwrap();
    for _ in 0..valid_action_count as usize {
        io::stdin().read_line(&mut input_line_2).unwrap();
    }
    (
        inputs[0].trim().parse::<i8>().unwrap(),
        inputs[1].trim().parse::<i8>().unwrap(),
    )
}

