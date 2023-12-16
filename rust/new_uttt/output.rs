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
        use super::cg_rand::Rng;
        use super::masks::local_to_global;
        use super::mcts::traits::GameState;
        use super::mcts::traits::GameResult;
        use super::game_action::Action;
        use super::global_board::GlobalBoard;
        use super::local_board::LocalBoards;
        use super::player::Player;
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
                new_state.player = new_state.player.other();
                let board_state = new_state.local_boards.set(
                    action.global(),
                    action.local(),
                    new_state.player,
                );
                new_state.result = new_state.global_states.set(action.global(), board_state);
                new_state.last_action = Some(*action);
                new_state.last_local_move = Some(action.local());
                new_state
            }
            fn simulate_game(mut self, rng: &mut Box<Rng>) -> UTTTResult {
                loop {
                    let action = self.random_move(rng);
                    match action {
                        Some(action) => {
                            self.player = self.player.other();
                            let board_state = self.local_boards.set(
                                action.global(),
                                action.local(),
                                self.player,
                            );
                            self.result = self.global_states.set(action.global(), board_state);
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
            pub fn random_move(&self, rng: &mut Box<Rng>) -> Option<Action> {
                let global = local_to_global(self.last_local_move.unwrap());
                if self.global_states.in_play(global) {
                    rng.choice(self.local_boards.local_moves(global).iter())
                        .map(|&local| Action::from_coords(global, local))
                } else {
                    rng.choice(self.global_states.playable_boards.iter()).map(
                        |&global| {
                            rng.choice(self.local_boards.local_moves(global).iter())
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
                    UTTTResult::Drawn => 0.5,
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
            expanded: Cell<bool>,
            pub wins: Cell<f64>,
            pub visits: Cell<f64>,
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
                    expanded: Cell::new(false),
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
                    expanded: Cell::new(false),
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
            pub fn uct(&self, visits: f64) -> f64 {
                self.wins.get() / self.visits.get() + C * (visits / self.visits.get())
            }
            pub fn select(
                self_ref: Rc<MctsNode<P, S, R, A>>,
                depth: usize,
            ) -> Rc<MctsNode<P, S, R, A>> {
                if depth == 0 {
                    return self_ref;
                }
                let uc = self_ref.unvisited_actions.borrow();
                let children = self_ref.children.borrow();
                let fully_expanded = self_ref.expanded.get();
                if fully_expanded && !children.is_empty() {
                    let visits = self_ref.visits.get().ln();
                    let selection = self_ref
                        .children
                        .borrow()
                        .values()
                        .reduce(|acc, node| if acc.uct(visits) > node.uct(visits) {
                            acc
                        } else {
                            node
                        })
                        .unwrap()
                        .clone();
                    MctsNode::select(selection, depth - 1)
                } else if !fully_expanded {
                    drop(uc);
                    drop(children);
                    let mut unvisited_actions = self_ref.unvisited_actions.borrow_mut();
                    let next_action = unvisited_actions.pop();
                    drop(unvisited_actions);
                    match next_action {
                        Some(action) => {
                            let mut children = self_ref.children.borrow_mut();
                            let state = self_ref.state.perform_action_copy(&action);
                            let child =
                                Rc::new(MctsNode::create_child(state, Rc::downgrade(&self_ref)));
                            children.insert(action.clone(), Rc::clone(&child));
                            child
                        }
                        None => {
                            self_ref.expanded.set(true);
                            MctsNode::select(self_ref, depth - 1)
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
                self.wins.set(
                    self.wins.get() +
                        self.state.current_player().reward(result),
                );
                if let Some(parent) = self.parent.upgrade() {
                    parent.backpropagate(result);
                }
            }
        }
    }
    pub mod tree {
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
                MctsTree { root: RefCell::new(node_ref) }
            }
            pub fn best_child(&self) -> Rc<MctsNode<P, S, R, A>> {
                self.root.borrow().best_child()
            }
            pub fn expand_tree(
                &self,
                begin: Instant,
                duration: time::Duration,
                rng: &mut Box<Rng>,
                depth: usize,
            ) {
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
                let child: Rc<MctsNode<P, S, R, A>> =
                    Rc::clone(&self.root.borrow().children.borrow().get(&action).unwrap());
                self.root.replace(child);
            }
        }
    }
    pub mod traits {
        use super::cg_rand::Rng;
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
            fn simulate_game(self, rng: &mut Box<Rng>) -> R;
            fn outcome(&self) -> R;
            fn playable(&self) -> bool;
        }
    }
    pub use super::*;
}
pub mod cg_rand {
    extern crate alloc ;
    use core::ops::Bound;
    use core::ops::RangeBounds;
    mod global_rng {
        use super::cg_rand::Rng;
        use std::cell::Cell;
        use std::ops::RangeBounds;
        const DEFAULT_RNG_SEED: u64 = 0xef6f79ed30ba75a;
        impl Default for Rng {
            #[inline]
            fn default() -> Rng {
                Rng::new()
            }
        }
        impl Rng {
            #[inline]
            pub fn new() -> Rng {
                try_with_rng(Rng::fork).unwrap_or_else(|_| Rng::with_seed(0x4d595df4d0f33173))
            }
        }
        thread_local ! { static RNG : Cell < Rng > = Cell :: new ( Rng ( random_seed ( ) . unwrap_or ( DEFAULT_RNG_SEED ) ) ) ; }
        #[inline]
        fn with_rng<R>(f: impl FnOnce(&mut Rng) -> R) -> R {
            RNG.with(|rng| {
                let current = rng.replace(Rng(0));
                let mut restore = RestoreOnDrop { rng, current };
                f(&mut restore.current)
            })
        }
        #[inline]
        fn try_with_rng<R>(f: impl FnOnce(&mut Rng) -> R) -> Result<R, std::thread::AccessError> {
            RNG.try_with(|rng| {
                let current = rng.replace(Rng(0));
                let mut restore = RestoreOnDrop { rng, current };
                f(&mut restore.current)
            })
        }
        struct RestoreOnDrop<'a> {
            rng: &'a Cell<Rng>,
            current: Rng,
        }
        impl Drop for RestoreOnDrop<'_> {
            fn drop(&mut self) {
                self.rng.set(Rng(self.current.0));
            }
        }
        #[inline]
        pub fn seed(seed: u64) {
            with_rng(|r| r.seed(seed));
        }
        #[inline]
        pub fn get_seed() -> u64 {
            with_rng(|r| r.get_seed())
        }
        #[inline]
        pub fn choice<I>(iter: I) -> Option<I::Item>
        where
            I: IntoIterator,
            I::IntoIter: ExactSizeIterator,
        {
            with_rng(|r| r.choice(iter))
        }
        #[inline]
        pub fn usize(range: impl RangeBounds<usize>) -> usize {
            with_rng(|r| r.usize(range))
        }
        fn random_seed() -> Option<u64> {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            use std::thread;
            use std::time::Instant;
            let mut hasher = DefaultHasher::new();
            Instant::now().hash(&mut hasher);
            thread::current().id().hash(&mut hasher);
            let hash = hasher.finish();
            Some((hash << 1) | 1)
        }
    }
    pub use super::*;
    pub use global_rng::*;
    #[derive(Debug, PartialEq, Eq)]
    pub struct Rng(u64);
    impl Clone for Rng {
        fn clone(&self) -> Rng {
            Rng::with_seed(self.0)
        }
    }
    impl Rng {
        #[inline]
        #[cfg(target_pointer_width = "32")]
        fn gen_u32(&mut self) -> u32 {
            self.gen_u64() as u32
        }
        #[inline]
        fn gen_u64(&mut self) -> u64 {
            let s = self.0.wrapping_add(0xA0761D6478BD642F);
            self.0 = s;
            let t = u128::from(s) * u128::from(s ^ 0xE7037ED1A0B428DB);
            (t as u64) ^ (t >> 64) as u64
        }
        #[inline]
        #[cfg(target_pointer_width = "128")]
        fn gen_u128(&mut self) -> u128 {
            (u128::from(self.gen_u64()) << 64) | u128::from(self.gen_u64())
        }
        #[inline]
        #[cfg(target_pointer_width = "32")]
        fn gen_mod_u32(&mut self, n: u32) -> u32 {
            let mut r = self.gen_u32();
            let mut hi = mul_high_u32(r, n);
            let mut lo = r.wrapping_mul(n);
            if lo < n {
                let t = n.wrapping_neg() % n;
                while lo < t {
                    r = self.gen_u32();
                    hi = mul_high_u32(r, n);
                    lo = r.wrapping_mul(n);
                }
            }
            hi
        }
        #[inline]
        fn gen_mod_u64(&mut self, n: u64) -> u64 {
            let mut r = self.gen_u64();
            let mut hi = mul_high_u64(r, n);
            let mut lo = r.wrapping_mul(n);
            if lo < n {
                let t = n.wrapping_neg() % n;
                while lo < t {
                    r = self.gen_u64();
                    hi = mul_high_u64(r, n);
                    lo = r.wrapping_mul(n);
                }
            }
            hi
        }
        #[inline]
        #[cfg(target_pointer_width = "128")]
        fn gen_mod_u128(&mut self, n: u128) -> u128 {
            let mut r = self.gen_u128();
            let mut hi = mul_high_u128(r, n);
            let mut lo = r.wrapping_mul(n);
            if lo < n {
                let t = n.wrapping_neg() % n;
                while lo < t {
                    r = self.gen_u128();
                    hi = mul_high_u128(r, n);
                    lo = r.wrapping_mul(n);
                }
            }
            hi
        }
    }
    #[inline]
    #[cfg(target_pointer_width = "32")]
    fn mul_high_u32(a: u32, b: u32) -> u32 {
        (((a as u64) * (b as u64)) >> 32) as u32
    }
    #[inline]
    fn mul_high_u64(a: u64, b: u64) -> u64 {
        (((a as u128) * (b as u128)) >> 64) as u64
    }
    #[inline]
    #[cfg(target_pointer_width = "128")]
    fn mul_high_u128(a: u128, b: u128) -> u128 {
        let a_lo = a as u64 as u128;
        let a_hi = (a >> 64) as u64 as u128;
        let b_lo = b as u64 as u128;
        let b_hi = (b >> 64) as u64 as u128;
        let carry = (a_lo * b_lo) >> 64;
        let carry = ((a_hi * b_lo) as u64 as u128 + (a_lo * b_hi) as u64 as u128 + carry) >> 64;
        a_hi * b_hi + ((a_hi * b_lo) >> 64) + ((a_lo * b_hi) >> 64) + carry
    }
    macro_rules ! rng_integer { ( $ t : tt , $ unsigned_t : tt , $ gen : tt , $ mod : tt , $ doc : tt ) => { # [ doc = $ doc ] # [ inline ] pub fn $ t ( & mut self , range : impl RangeBounds <$ t > ) -> $ t { let panic_empty_range = || { panic ! ( "empty range: {:?}..{:?}" , range . start_bound ( ) , range . end_bound ( ) ) } ; let low = match range . start_bound ( ) { Bound :: Unbounded => core ::$ t :: MIN , Bound :: Included ( & x ) => x , Bound :: Excluded ( & x ) => x . checked_add ( 1 ) . unwrap_or_else ( panic_empty_range ) , } ; let high = match range . end_bound ( ) { Bound :: Unbounded => core ::$ t :: MAX , Bound :: Included ( & x ) => x , Bound :: Excluded ( & x ) => x . checked_sub ( 1 ) . unwrap_or_else ( panic_empty_range ) , } ; if low > high { panic_empty_range ( ) ; } if low == core ::$ t :: MIN && high == core ::$ t :: MAX { self .$ gen ( ) as $ t } else { let len = high . wrapping_sub ( low ) . wrapping_add ( 1 ) ; low . wrapping_add ( self .$ mod ( len as $ unsigned_t as _ ) as $ t ) } } } ; }
    impl Rng {
        #[inline]
        #[must_use = "this creates a new instance of `Rng`; if you want to initialize the thread-local generator, use `fastrand::seed()` instead"]
        pub fn with_seed(seed: u64) -> Self {
            let mut rng = Rng(0);
            rng.seed(seed);
            rng
        }
        #[inline]
        #[must_use = "this creates a new instance of `Rng`"]
        pub fn fork(&mut self) -> Self {
            Rng::with_seed(self.gen_u64())
        }
        #[inline]
        pub fn seed(&mut self, seed: u64) {
            self.0 = seed;
        }
        #[inline]
        pub fn get_seed(&self) -> u64 {
            self.0
        }
        #[inline]
        pub fn choice<I>(&mut self, iter: I) -> Option<I::Item>
        where
            I: IntoIterator,
            I::IntoIter: ExactSizeIterator,
        {
            let mut iter = iter.into_iter();
            let len = iter.len();
            if len == 0 {
                return None;
            }
            let index = self.usize(0..len);
            iter.nth(index)
        }
        #[cfg(target_pointer_width = "32")]
        rng_integer!(
            usize,
            usize,
            gen_u32,
            gen_mod_u32,
            "Generates a random `usize` in the given range."
        );
        #[cfg(target_pointer_width = "64")]
        rng_integer!(
            usize,
            usize,
            gen_u64,
            gen_mod_u64,
            "Generates a random `usize` in the given range."
        );
        #[cfg(target_pointer_width = "128")]
        rng_integer!(
            usize,
            usize,
            gen_u128,
            gen_mod_u128,
            "Generates a random `usize` in the given range."
        );
    }
}
use std::io;
use std::rc::Rc;
use std::time;
use game::game_action::Action;
use game::game_state::State;
use game::masks::LOCAL_MOVES;
use mcts::tree::MctsTree;
use cg_rand::Rng;
use mcts::traits::GameState;
use mcts::node::MctsNode;
fn main() {
    codingame();
}
fn perf_test() {
    let mut rng = Box::new(cg_rand::Rng::new());
    let mut game = State::default();
    let mcts = MctsTree::new(game);
    let mut action = Action::from_coords(4, LOCAL_MOVES[4]);
    let root_ref = mcts.root.borrow();
    let state = root_ref.state.perform_action_copy(&action);
    let next = Rc::new(MctsNode::create_child(state, Rc::downgrade(&root_ref)));
    drop(root_ref);
    mcts.root.replace(next);
    let first_turn_begin = time::Instant::now();
    let first_duration = time::Duration::new(0, 999999995);
    mcts.expand_tree(first_turn_begin, first_duration, &mut rng, 81);
    println!("4 4");
    let duration = time::Duration::new(0, 99000000);
    loop {
        let begin = time::Instant::now();
        mcts.expand_tree(begin, duration, &mut rng, 5);
        let child = mcts.best_child();
        game = child.state.clone();
        action = game.last_action.unwrap();
        mcts.root.replace(child);
        action.print();
        if !game.playable() {
            println!("{:?}", game.outcome());
            return;
        }
        let opp = mcts.best_child();
        game = opp.state.clone();
        action = game.last_action.unwrap();
        mcts.root.replace(opp);
        action.print();
        if !game.playable() {
            println!("{:?}", game.outcome());
            return;
        }
    }
}
fn codingame() {
    let mut rng: Box<Rng> = Box::new(cg_rand::Rng::new());
    let inputs = read_input();
    let game = State::default();
    let mcts = MctsTree::new(game);
    let mut action: Action;
    let first_turn_begin = time::Instant::now();
    let first_duration = time::Duration::new(0, 999999900);
    if inputs.0 < 0 {
        action = Action::from_coords(4, LOCAL_MOVES[4]);
        let root_ref = mcts.root.borrow();
        let state = root_ref.state.perform_action_copy(&action);
        let next = Rc::new(MctsNode::create_child(state, Rc::downgrade(&root_ref)));
        drop(root_ref);
        mcts.root.replace(next);
        mcts.expand_tree(first_turn_begin, first_duration, &mut rng, 10);
        println!("4 4");
    } else {
        let opponent_row = inputs.0;
        let opponent_col = inputs.1;
        action = Action::from_coords(
            (opponent_col / 3 + (opponent_row / 3) * 3) as usize,
            LOCAL_MOVES[(opponent_col % 3 + (opponent_row % 3) * 3) as usize],
        );
        let root_ref = mcts.root.borrow();
        let state = root_ref.state.perform_action_copy(&action);
        let next = Rc::new(MctsNode::create_child(state, Rc::downgrade(&root_ref)));
        drop(root_ref);
        mcts.root.replace(next);
        mcts.expand_tree(first_turn_begin, first_duration, &mut rng, 10);
        let child = mcts.root.borrow().best_child();
        action = child.state.last_action.unwrap();
        mcts.root.replace(child);
        action.print();
    }
    let turn_duration = time::Duration::new(0, 99999900);
    loop {
        let begin = time::Instant::now();
        let inputs = read_input();
        let opponent_row = inputs.0;
        let opponent_col = inputs.1;
        let opp_move = Action::from_coords(
            (opponent_col / 3 + (opponent_row / 3) * 3) as usize,
            LOCAL_MOVES[(opponent_col % 3 + (opponent_row % 3) * 3) as usize],
        );
        mcts.move_down(opp_move);
        mcts.expand_tree(begin, turn_duration, &mut rng, 10);
        let child = mcts.best_child();
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

