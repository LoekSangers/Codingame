extern crate rand;
use rand::prelude::{IteratorRandom, ThreadRng};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::vec::Vec;
use std::{io, time};

// based on: https://github.com/nelhage/ultimattt/blob/master/src/lib/game.rs
// Removed SIMD as this is not included in safe rust which is needed for Codingame
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    InPlay,
    Drawn,
    Won(Player),
}

#[allow(clippy::unusual_byte_groupings)]
//#[inline]
pub(in crate) const fn win_masks_for_move(local: usize) -> &'static [usize] {
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

#[allow(clippy::unusual_byte_groupings)]
pub(in crate) const LOCAL_MOVES: &[usize] = &[
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

#[allow(clippy::unusual_byte_groupings)]
//#[inline]
pub(in crate) const fn local_to_global(local: usize) -> usize {
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

#[allow(clippy::unusual_byte_groupings)]
const BOARD_MASK: usize = 0b111_111_111;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate) struct LocalBoards {
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
    //#[inline]
    fn local_moves(&self, global: usize) -> &Vec<usize> {
        &self.legal_moves[global]
    }

    //#[inline]
    fn set(&mut self, global: usize, local: usize, player: Player) -> GameState {
        self.legal_moves[global].retain(|&loc| loc != local);
        match player {
            Player::O => {
                self.boards_o[global] |= local;
                if win_masks_for_move(local)
                    .iter()
                    .any(|&win_mask| self.boards_o[global] & win_mask == win_mask)
                {
                    return GameState::Won(Player::O);
                }
            }
            Player::X => {
                self.boards_x[global] |= local;
                if win_masks_for_move(local)
                    .iter()
                    .any(|&win_mask| self.boards_x[global] & win_mask == win_mask)
                {
                    return GameState::Won(Player::X);
                }
            }
        }
        if (self.boards_o[global] | self.boards_x[global]) == BOARD_MASK {
            GameState::Drawn
        } else {
            GameState::InPlay
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate) struct GlobalStates {
    // LSB first; drawn if both x&y
    x: usize,
    o: usize,
    x_won: u16,
    o_won: u16,
    playable_boards: Vec<usize>,
}

impl Default for GlobalStates {
    //#[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalStates {
    //#[inline]
    pub fn new() -> GlobalStates {
        GlobalStates {
            x: 0,
            o: 0,
            x_won: 0,
            o_won: 0,
            playable_boards: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        }
    }

    //#[inline]
    pub(in crate) fn in_play(&self, board: usize) -> bool {
        self.playable_boards.contains(&board)
    }

    //#[inline]
    fn set(&mut self, board: usize, state: GameState) -> GameState {
        let bit = 1_usize << board;
        match state {
            GameState::Drawn => {
                self.playable_boards.retain(|&x| x != board);

                self.x |= bit;
                self.o |= bit;

                if self.playable_boards.is_empty() {
                    match self.x_won.cmp(&self.o_won) {
                        Ordering::Greater => GameState::Won(Player::X),
                        Ordering::Less => GameState::Won(Player::O),
                        Ordering::Equal => GameState::Drawn,
                    }
                } else {
                    GameState::InPlay
                }
            }
            GameState::InPlay => GameState::InPlay,
            GameState::Won(Player::X) => {
                self.playable_boards.retain(|&x| x != board);

                self.x_won += 1;
                self.x |= bit;

                if win_masks_for_move(bit)
                    .iter()
                    .any(|&win_mask| self.x & win_mask == win_mask)
                {
                    return GameState::Won(Player::X);
                }

                if self.playable_boards.is_empty() {
                    match self.x_won.cmp(&self.o_won) {
                        Ordering::Greater => GameState::Won(Player::X),
                        Ordering::Less => GameState::Won(Player::O),
                        Ordering::Equal => GameState::Drawn,
                    }
                } else {
                    GameState::InPlay
                }
            }
            GameState::Won(Player::O) => {
                self.playable_boards.retain(|&x| x != board);

                self.o_won += 1;
                self.o |= bit;

                if win_masks_for_move(bit)
                    .iter()
                    .any(|&win_mask| self.o & win_mask == win_mask)
                {
                    return GameState::Won(Player::O);
                }

                if self.playable_boards.is_empty() {
                    match self.x_won.cmp(&self.o_won) {
                        Ordering::Greater => GameState::Won(Player::X),
                        Ordering::Less => GameState::Won(Player::O),
                        Ordering::Equal => GameState::Drawn,
                    }
                } else {
                    GameState::InPlay
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Move {
    bits: usize,
}

impl Move {
    //#[inline]
    pub fn from_coords(global: usize, local: usize) -> Self {
        Self {
            bits: (global << 9 | local),
        }
    }
    //#[inline]
    pub fn global(self) -> usize {
        self.bits >> 9
    }
    //#[inline]
    pub fn local(self) -> usize {
        self.bits & BOARD_MASK
    }

    //#[inline]
    pub fn print(&self) {
        let row = self.global() / 3 * 3 + local_to_global(self.local()) / 3;
        let col = self.global() % 3 * 3 + local_to_global(self.local()) % 3;
        println!("{} {}", row, col);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub(in crate) player: Player,
    pub(in crate) last_local_move: Option<usize>,
    pub(in crate) local_boards: LocalBoards,
    pub(in crate) global_states: GlobalStates,
    pub(in crate) game_state: GameState,
}

impl Default for Game {
    //#[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    //#[inline]
    pub fn new() -> Game {
        Game {
            player: Player::X,
            last_local_move: None,
            local_boards: Default::default(),
            global_states: Default::default(),
            game_state: GameState::InPlay,
        }
    }

    //#[inline]
    pub fn inplace_move(&mut self, m: &Move) -> GameState {
        let board_state = self.local_boards.set(m.global(), m.local(), self.player);

        self.game_state = self.global_states.set(m.global(), board_state);
        self.player = self.player.other();
        self.last_local_move = Some(m.local());

        board_state
    }

    //#[inline]
    pub fn all_moves(&self) -> Vec<Move> {
        match self.last_local_move {
            Some(last_local) => {
                let global = local_to_global(last_local);
                if self.global_states.in_play(global) {
                    self.local_boards
                        .local_moves(global)
                        .iter()
                        .map(|&local| Move::from_coords(global, local))
                        .collect()
                } else {
                    self.global_states
                        .playable_boards
                        .iter()
                        .flat_map(|&global| {
                            self.local_boards
                                .local_moves(global)
                                .iter()
                                .map(move |&local| Move::from_coords(global, local))
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
                        .map(move |&local| Move::from_coords(global, local))
                })
                .collect(),
        }
    }

    //#[inline]
    fn random_move(&self, rng: &mut ThreadRng) -> Move {
        let global = local_to_global(self.last_local_move.unwrap());
        if self.global_states.in_play(global) {
            self.local_boards
                .local_moves(global)
                .iter()
                .choose(rng)
                .map(|&local| Move::from_coords(global, local))
                .unwrap()
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
                        .map(|&local| Move::from_coords(global, local))
                        .unwrap()
                })
                .unwrap()
        }
    }

    //#[inline]
    pub fn random_playout(&mut self, rng: &mut ThreadRng) {
        while self.game_state == GameState::InPlay {
            self.inplace_move(&self.random_move(rng));
        }
    }

    //#[inline]
    pub fn player(&self) -> Player {
        self.player
    }
}

#[derive(Debug)]
pub struct Node {
    children: HashMap<Move, Node>,
    unvisited_moves: Vec<Move>,
    visits: f32,
    wins: f32,
}

impl Default for Node {
    //#[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Node {
    //#[inline]
    fn score(&self) -> f32{
        self.wins / self.visits + 0.6 * (self.visits.ln() / self.visits).sqrt()
    }
    //#[inline]
    pub fn new() -> Node {
        Node {
            children: HashMap::new(),
            unvisited_moves: vec![],
            visits: 0.0,
            wins: 0.0,
        }
    }
    //#[inline]
    pub fn run(&mut self, game: &mut Game, run_time_nano: u32, rng: &mut ThreadRng) {
        let begin = time::Instant::now();
        let mut count = 0;
        let duration = time::Duration::new(0, run_time_nano);
        while begin.elapsed() < duration {
            self.iteration(&mut game.clone(), rng);
            count += 1;
        }
        eprintln!("{}", count);
    }

    pub fn iteration(&mut self, game: &mut Game, rng: &mut ThreadRng) -> (f32, f32) {
        let rewards = if !self.unvisited_moves.is_empty() {
            //expand
            let m = self.unvisited_moves.pop().unwrap();
            game.inplace_move(&m);
            let mut child = Node {
                unvisited_moves: game.all_moves(),
                ..Default::default()
            };

            let current_player = game.player();
            game.random_playout(rng);

            let rewards = match current_player {
                Player::X => match game.game_state {
                    GameState::Won(Player::X) => (0., 1.),
                    GameState::Won(Player::O) => (1., 0.),
                    GameState::Drawn => (0.5, 0.5),
                    _ => (0., 0.),
                },
                Player::O => match game.game_state {
                    GameState::Won(Player::O) => (0., 1.),
                    GameState::Won(Player::X) => (1., 0.),
                    GameState::Drawn => (0.5, 0.5),
                    _ => (0., 0.),
                },
            };

            child.visits += 1.;
            child.wins += rewards.0;
            self.children.insert(m, child);
            (rewards.1, rewards.0)
        } else if !self.children.is_empty() {
            //continue selection
            let (m, child) = self.select_best_child();
            game.inplace_move(m);

            child.iteration(game, rng)
        } else {
            //it was a leaf to begin with
            let current_player = game.player();
            match current_player {
                Player::X => match game.game_state {
                    GameState::Won(Player::X) => (0., 1.),
                    GameState::Won(Player::O) => (1., 0.),
                    GameState::Drawn => (0.5, 0.5),
                    _ => (0., 0.),
                },
                Player::O => match game.game_state {
                    GameState::Won(Player::O) => (0., 1.),
                    GameState::Won(Player::X) => (1., 0.),
                    GameState::Drawn => (0.5, 0.5),
                    _ => (0., 0.),
                },
            }
        };

        //backpropagate
        self.visits += 1.;
        self.wins += rewards.0;
        (rewards.1, rewards.0)
    }

    //#[inline]
    pub fn select_best_child(&mut self) -> (&Move, &mut Node) {
        self
            .children
            .iter_mut()
            .reduce(|acc, curr| {
                if curr.1.score() > acc.1.score() {
                    curr
                }else{
                    acc
                }
            }).unwrap()
    }

    //#[inline]
    pub fn select_best_move(&mut self) -> (&Move, &mut Node) {
        self
            .children
            .iter_mut()
            .reduce(|acc, curr| {
                if curr.1.wins / curr.1.visits > acc.1.wins / acc.1.visits {
                    curr
                }else{
                    acc
                }
            }).unwrap()
    }
}

#[allow(dead_code)]
//#[inline]
fn read_input() -> (String, String) {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(' ').collect::<Vec<_>>();
    let mut input_line_2 = String::new();
    io::stdin().read_line(&mut input_line_2).unwrap();
    let valid_action_count = input_line_2.trim().parse::<i32>().unwrap();
    for _ in 0..valid_action_count as usize {
        io::stdin().read_line(&mut input_line_2).unwrap();
    }

    (inputs[0].to_string(), inputs[1].to_string())
}

#[allow(dead_code)]
//#[inline]
fn codingame() {
    let mut rng = rand::thread_rng();
    let inputs = read_input();

    let mut game = Game::default();
    let mut root: &mut Node = &mut Node::new();
    let mut m: &Move;

    match inputs.0.chars().next() {
        Some('-') => {
            game.inplace_move(&Move::from_coords(4, LOCAL_MOVES[4]));
            root.unvisited_moves = game.all_moves();
            root.run(&mut game, 999999995, &mut rng);
            println!("4 4");
        }
        Some(_) => {
            let opponent_row = inputs.0.parse::<i8>().unwrap();
            let opponent_col = inputs.1.parse::<i8>().unwrap();

            game.inplace_move(&Move::from_coords(
                (opponent_col / 3 + (opponent_row / 3) * 3) as usize,
                LOCAL_MOVES[(opponent_col % 3 + (opponent_row % 3) * 3) as usize],
            ));
            root.unvisited_moves = game.all_moves();
            root.run(&mut game, 999999997, &mut rng);

            let child = root.select_best_move();
            m = child.0;
            root = child.1;

            game.inplace_move(m);
            m.print();
        }
        _ => panic!(),
    }

    // game loop
    loop {
        let inputs = read_input();
        let opponent_row = inputs.0.trim().parse::<i8>().unwrap();
        let opponent_col = inputs.1.trim().parse::<i8>().unwrap();
        let opp_move = Move::from_coords(
            (opponent_col / 3 + (opponent_row / 3) * 3) as usize,
            LOCAL_MOVES[(opponent_col % 3 + (opponent_row % 3) * 3) as usize],
        );
        game.inplace_move(&opp_move);
        root = root.children.get_mut(&opp_move).unwrap();
        root.run(&mut game, 99999900, &mut rng);

        let child = root.select_best_move();
        m = child.0;
        root = child.1;
        game.inplace_move(m);
        m.print();
    }
}

fn perf_test() {
    let mut rng = rand::thread_rng();
    let mut root = &mut Node::new();
    let mut m: &Move;

    let mut game = Game::default();

    game.inplace_move(&Move::from_coords(4, 4));
    root.unvisited_moves = game.all_moves();
    root.run(&mut game, 999999990, &mut rng);
    println!("4 4");

    // game loop
    loop {
        root.run(&mut game, 99999900, &mut rng);

        let child = root.select_best_move();
        m = child.0;
        root = child.1;

        game.inplace_move(m);
        m.print();
    }
}

fn main() {
    codingame();
}
