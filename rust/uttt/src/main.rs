use rand::prelude::{IteratorRandom, ThreadRng};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::vec::Vec;
use std::{io, time};

#[derive(PartialEq)]
struct NonNan(f32);

impl NonNan {
    fn new(val: f32) -> Option<NonNan> {
        if val.is_nan() {
            None
        } else {
            Some(NonNan(val))
        }
    }
}

impl Eq for NonNan {}

impl PartialOrd for NonNan {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for NonNan {
    fn cmp(&self, other: &NonNan) -> Ordering {
        self.partial_cmp(other).unwrap()
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if self > other {
            self
        } else {
            other
        }
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if self < other {
            self
        } else {
            other
        }
    }

    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
    {
        assert!(min <= max);
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

// based on: https://github.com/nelhage/ultimattt/blob/master/src/lib/game.rs
// Removed SIMD as this is not included in safe rust which is needed for Codingame
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Player {
    X,
    O,
}

impl Player {
    pub fn other(&self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Player::X => "X",
            Player::O => "O",
        }
    }

    pub fn as_bit(&self) -> usize {
        match self {
            Player::X => 0,
            Player::O => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellState {
    Empty,
    Played(Player),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    InPlay,
    Drawn,
    Won(Player),
}

impl GameState {
    pub fn terminal(&self) -> bool {
        matches!(self, GameState::Drawn | GameState::Won(_))
    }
}

#[allow(clippy::unusual_byte_groupings)]
pub(in crate) const WIN_MASKS: &[u32] = &[
    0b000_000_111,
    0b000_111_000,
    0b111_000_000,
    0b001_001_001,
    0b010_010_010,
    0b100_100_100,
    0b001_010_100,
    0b100_010_001,
];

#[allow(clippy::unusual_byte_groupings)]
const BOARD_MASK: u32 = 0b111_111_111;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    // These are each a packed [u9; 3] containing a bitmask for the
    // respective player's states. The low bits store index 0.
    x: u32,
    o: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate) struct LocalBoards {
    rows: [Row; 3],
}

impl Default for LocalBoards {
    fn default() -> Self {
        LocalBoards {
            rows: [Row { x: 0, o: 0 }, Row { x: 0, o: 0 }, Row { x: 0, o: 0 }],
        }
    }
}

impl LocalBoards {
    fn at(&self, global: usize, local: usize) -> CellState {
        let row = &self.rows[global / 3];
        let idx = (global % 3) * 9 + local;
        if (row.x >> idx) & 1 == 1 {
            return CellState::Played(Player::X);
        }
        if (row.o >> idx) & 1 == 1 {
            return CellState::Played(Player::O);
        }
        CellState::Empty
    }

    fn set(&mut self, global: usize, local: usize, who: Player) {
        let mut row = &mut self.rows[global / 3];
        let idx = (global % 3) * 9 + local;
        let bit = 1 << idx;
        match who {
            Player::X => {
                row.x |= bit;
            }
            Player::O => {
                row.o |= bit;
            }
        }
    }

    fn check_winner(&self, global: usize, player: Player) -> GameState {
        let row = &self.rows[global / 3];
        let shift = 9 * (global % 3);
        let mask = match player {
            Player::X => row.x >> shift,
            Player::O => row.o >> shift,
        };

        if WIN_MASKS
            .iter()
            .any(|&win_mask| mask & win_mask == win_mask)
        {
            return GameState::Won(player);
        }

        if ((row.x | row.o) >> shift) & BOARD_MASK == BOARD_MASK {
            GameState::Drawn
        } else {
            GameState::InPlay
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub(in crate) struct GlobalStates {
    // LSB first; drawn if both x&y
    x: u16,
    o: u16,
}

impl GlobalStates {
    pub(in crate) fn xbits(&self) -> u32 {
        (self.x & !self.o) as u32
    }
    pub(in crate) fn obits(&self) -> u32 {
        (self.o & !self.x) as u32
    }
    pub(in crate) fn drawbits(&self) -> u32 {
        (self.x & self.o) as u32
    }
    pub(in crate) fn donebits(&self) -> u32 {
        (self.x | self.o) as u32
    }
    pub(in crate) fn playerbits(&self, player: Player) -> u32 {
        match player {
            Player::X => self.xbits(),
            Player::O => self.obits(),
        }
    }
    pub(in crate) fn in_play(&self, board: usize) -> bool {
        (self.donebits() & 1 << board) == 0
    }

    fn check_winner(&self, player: Player) -> GameState {
        let mask = self.playerbits(player);
        if WIN_MASKS
            .iter()
            .any(|&win_mask| mask & win_mask == win_mask)
        {
            return GameState::Won(player);
        }

        if self.donebits() == BOARD_MASK {
            GameState::Drawn
        } else {
            GameState::InPlay
        }
    }

    fn at(&self, board: usize) -> GameState {
        let bit = 1 << board;
        if self.xbits() & bit != 0 {
            GameState::Won(Player::X)
        } else if self.obits() & bit != 0 {
            GameState::Won(Player::O)
        } else if self.drawbits() & bit != 0 {
            GameState::Drawn
        } else {
            GameState::InPlay
        }
    }

    fn set(&mut self, board: usize, state: GameState) {
        let bit = 1_u16 << board;
        match state {
            GameState::Drawn => {
                self.x |= bit;
                self.o |= bit;
            }
            GameState::InPlay => {}
            GameState::Won(Player::X) => self.x |= bit,
            GameState::Won(Player::O) => self.o |= bit,
        };
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Move {
    bits: u8,
}

impl Move {
    pub fn from_coords(global: u8, local: u8) -> Self {
        Self {
            bits: (global << 4 | local) as u8,
        }
    }

    pub fn global(self) -> usize {
        (self.bits >> 4) as usize
    }

    pub fn local(self) -> usize {
        (self.bits & 0x0f) as usize
    }

    pub fn none() -> Self {
        Move { bits: 0xff }
    }

    pub fn is_none(self) -> bool {
        self.bits == 0xff
    }

    pub fn is_some(self) -> bool {
        !self.is_none()
    }

    pub fn bits(self) -> u8 {
        self.bits
    }

    pub fn print(&self) {
        let row = self.global() / 3 * 3 + self.local() / 3;
        let col = self.global() % 3 * 3 + self.local() % 3;
        println!("{} {}", row, col);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub(in crate) player: Player,
    pub(in crate) last_move: Option<Move>,
    pub(in crate) local_boards: LocalBoards,
    pub(in crate) global_states: GlobalStates,
    pub(in crate) game_state: GameState,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Game {
        Game {
            player: Player::X,
            last_move: None,
            local_boards: Default::default(),
            global_states: Default::default(),
            game_state: GameState::InPlay,
        }
    }

    pub fn inplace_move(&mut self, m: Move) {
        self.local_boards.set(m.global(), m.local(), self.player);

        let board_state = self.local_boards.check_winner(m.global(), self.player);
        self.global_states.set(m.global(), board_state);

        self.game_state = self.global_states.check_winner(self.player);
        self.player = self.player.other();
        self.last_move = Some(m);
    }

    pub fn all_moves(&self) -> Vec<Move> {
        match self.last_move {
            Some(m) => {
                let global = m.local();
                if self.global_states.in_play(global) {
                    (0..=8_u8)
                        .filter(|local| {
                            self.local_boards.at(global, *local as usize) == CellState::Empty
                        })
                        .map(|local| Move::from_coords(global as u8, local))
                        .collect()
                } else {
                    (0..=8_u8)
                        .filter(|global| self.global_states.in_play(*global as usize))
                        .flat_map(|global| {
                            (0..=8_u8)
                                .filter(move |local| {
                                    self.local_boards.at(global as usize, *local as usize)
                                        == CellState::Empty
                                })
                                .map(move |local| Move::from_coords(global, local))
                        })
                        .collect()
                }
            }
            _ => (0..=8_u8)
                .filter(|global| self.global_states.in_play(*global as usize))
                .flat_map(|global| {
                    (0..=8_u8)
                        .filter(move |local| {
                            self.local_boards.at(global as usize, *local as usize)
                                == CellState::Empty
                        })
                        .map(move |local| Move::from_coords(global, local))
                })
                .collect(),
        }
    }

    fn random_move(&self, rng: &mut ThreadRng) -> Move {
        let global = self.last_move.unwrap().local();
        // let mut rng = rand::thread_rng();

        if self.global_states.in_play(global) {
            (0..=8_u8)
                .filter(|local| self.local_boards.at(global, *local as usize) == CellState::Empty)
                .choose(rng)
                .map(|local| Move::from_coords(global as u8, local))
                .unwrap()
        } else {
            (0..=8_u8)
                .filter(|global| self.global_states.in_play(*global as usize))
                .choose(rng)
                .map(|global| {
                    (0..=8_u8)
                        .filter(move |local| {
                            self.local_boards.at(global as usize, *local as usize)
                                == CellState::Empty
                        })
                        .choose(rng)
                        .map(|local| Move::from_coords(global as u8, local))
                        .unwrap()
                })
                .unwrap()
        }
    }

    pub fn random_playout(&mut self) -> (f32, f32) {
        let mut rng = rand::thread_rng();
        while self.game_state == GameState::InPlay {
            self.inplace_move(self.random_move(&mut rng))
        }
        self.reward()
    }

    pub fn reward(&self) -> (f32, f32) {
        match self.game_state {
            GameState::Won(Player::X) => (1., 0.),
            GameState::Won(Player::O) => (0., 1.),
            _ => (0., 0.),
        }
    }

    pub fn game_state(&self) -> GameState {
        self.game_state
    }

    pub fn game_over(&self) -> bool {
        !matches!(self.game_state, GameState::InPlay)
    }

    pub fn player(&self) -> Player {
        self.player
    }

    pub fn board_to_play(&self) -> Option<usize> {
        self.last_move.map(|m| m.local())
    }

    pub fn at(&self, board: usize, cell: usize) -> CellState {
        self.local_boards.at(board, cell)
    }

    pub fn board_state(&self, board: usize) -> GameState {
        self.global_states.at(board)
    }

    pub fn open_boards(&self) -> u8 {
        (!self.global_states.donebits() & BOARD_MASK).count_ones() as u8
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
    fn default() -> Self {
        Self::new(&Game::default())
    }
}

impl Node {
    pub fn new(game: &Game) -> Node {
        Node {
            children: HashMap::new(),
            unvisited_moves: game.all_moves(),
            visits: 0.0,
            wins: 0.0,
        }
    }

    pub fn run(&mut self, game: &mut Game, run_time_nano: u32) {
        let begin = time::Instant::now();
        let mut count = 0;
        while begin.elapsed() < time::Duration::new(0, run_time_nano) {
            self.iteration(&mut game.clone());
            count += 1;
        }
        eprintln!("{}", count);
    }

    pub fn iteration(&mut self, game: &mut Game) -> (f32, f32) {
        let rewards = if !self.unvisited_moves.is_empty() {
            //expand
            let m = self.unvisited_moves.pop().unwrap();
            game.inplace_move(m);
            let mut child = Node {
                unvisited_moves: game.all_moves(),
                ..Default::default()
            };
            let p = game.player();

            let mut rewards = game.random_playout();
            if p == Player::X {
                rewards = (rewards.1, rewards.0);
            }
            child.visits += 1.;
            child.wins += rewards.0;
            self.children.insert(m, child);
            (rewards.1, rewards.0)
        } else if !self.children.is_empty() {
            //continue selection
            let (m, child) = self.select_best_child(1.41);
            game.inplace_move(m);

            child.iteration(game)
        } else {
            //it was a leaf to begin with, should not really happen
            game.reward()
        };

        //backpropagate
        self.visits += 1.;
        self.wins += rewards.0;
        (rewards.1, rewards.0)
    }

    pub fn select_child(&mut self, m: Move) -> Option<&mut Node> {
        self.children.get_mut(&m)
    }

    pub fn select_best_child(&mut self, c: f32) -> (Move, &mut Node) {
        let visits = self.visits.ln();
        let (m, node) = self
            .children
            .iter_mut()
            .max_by_key(|(_, child)| {
                NonNan::new(child.wins / child.visits + c * (visits / child.visits).sqrt())
            })
            .unwrap();
        (*m, node)
    }
}

#[allow(dead_code)]
fn read_input() -> (String, String) {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(' ').collect::<Vec<_>>();
    let mut input_line_2 = String::new();
    io::stdin().read_line(&mut input_line_2).unwrap();
    let valid_action_count = parse_input!(input_line_2, i32);
    for _ in 0..valid_action_count as usize {
        io::stdin().read_line(&mut input_line_2).unwrap();
    }

    (inputs[0].to_string(), inputs[1].to_string())
}

#[allow(dead_code)]
fn codingame() {
    let inputs = read_input();

    let mut game = Game::default();
    let mut root: &mut Node = &mut Node::new(&game);
    let mut m: Move;

    match inputs.0.chars().next() {
        Some('-') => {
            game.inplace_move(Move::from_coords(4, 4));
            root.unvisited_moves = game.all_moves();
            root.run(&mut game, 900000000);
            println!("4 4");
        }
        Some(_) => {
            let opponent_row = parse_input!(inputs.0, i8);
            let opponent_col = parse_input!(inputs.1, i8);

            game.inplace_move(Move::from_coords(
                (opponent_col / 3 + (opponent_row / 3) * 3) as u8,
                (opponent_col % 3 + (opponent_row % 3) * 3) as u8,
            ));
            root.unvisited_moves = game.all_moves();
            root.run(&mut game, 900000000);

            let child = root.select_best_child(0.0);
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
        let opponent_row = parse_input!(inputs.0, i8);
        let opponent_col = parse_input!(inputs.1, i8);
        let opp_move = Move::from_coords(
            (opponent_col / 3 + (opponent_row / 3) * 3) as u8,
            (opponent_col % 3 + (opponent_row % 3) * 3) as u8,
        );
        game.inplace_move(opp_move);
        root = root.select_child(opp_move).unwrap();
        root.run(&mut game, 90000000);

        let child = root.select_best_child(0.0);
        m = child.0;
        root = child.1;
        game.inplace_move(m);
        m.print();
    }
}

#[allow(dead_code)]
fn perf_test() {
    let mut root = &mut Node::new(&Game::default());
    let mut m: Move;

    let mut game = Game::default();

    game.inplace_move(Move::from_coords(4, 4));
    root.run(&mut game, 900000000);
    println!("4 4");

    // game loop
    loop {
        root.run(&mut game, 90000000);

        let child = root.select_best_child(0.0);
        m = child.0;
        root = child.1;

        game.inplace_move(m);
        m.print();
    }
}

fn main() {
    perf_test();
}
