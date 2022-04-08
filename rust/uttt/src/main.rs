use std::io;

use std::vec::Vec;

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
        match self {
            GameState::Drawn | GameState::Won(_) => true,
            _ => false,
        }
    }
}

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

const BOARD_MASK: u32 = 0b111_111_111;

#[derive(Clone, Debug)]
pub struct LocalBoard([CellState; 9]);

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
        return CellState::Empty;
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

    pub(in crate) fn mask(&self, global: usize) -> u32 {
        let row = &self.rows[global / 3];
        (row.o | row.x) >> (9 * (global % 3)) & BOARD_MASK
    }

    fn check_winner(&self, global: usize, player: Player) -> GameState {
        let row = &self.rows[global / 3];
        let shift = 9 * (global % 3);
        let mask = match player {
            Player::X => row.x >> shift,
            Player::O => row.o >> shift,
        };

        if WIN_MASKS.iter().any(|&win_mask| mask & win_mask == win_mask) {
            return GameState::Won(player);
        }

        if ((row.x | row.o) >> shift) & BOARD_MASK == BOARD_MASK {
            GameState::Drawn
        } else {
            GameState::InPlay
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate) struct GlobalStates {
    // LSB first; drawn if both x&y
    x: u16,
    o: u16,
}

impl Default for GlobalStates {
    fn default() -> Self {
        GlobalStates { x: 0, o: 0 }
    }
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
        if WIN_MASKS.iter().any(|&win_mask| mask & win_mask == win_mask) {
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
            GameState::InPlay => {
                return;
            }
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
    pub fn from_coords(global: usize, local: usize) -> Self {
        return Self {
            bits: (global << 4 | local) as u8,
        };
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
}

#[derive(Debug, PartialEq, Eq)]
pub enum MoveError {
    WrongBoard,
    WonBoard,
    OutOfBounds,
    NotEmpty,
    GameOver,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub(in crate) next_player: Player,
    pub(in crate) next_board: Option<u8>,
    pub(in crate) local_boards: LocalBoards,
    pub(in crate) global_states: GlobalStates,
    pub(in crate) game_state: GameState,
    hash: u64,
}

impl Game {
    pub fn new() -> Game {
        Game {
            next_player: Player::X,
            next_board: None,
            local_boards: Default::default(),
            global_states: Default::default(),
            game_state: GameState::InPlay,
            hash: 0,
        }
    }

    pub fn inplace_move(&mut self, m: Move) -> Result<(), MoveError> {
        self.local_boards.set(m.global(), m.local(), self.next_player);
        
        let board_state = self.local_boards.check_winner(m.global(), self.next_player);
        self.global_states.set(m.global(), board_state);
        
        if self.global_states.in_play(m.local()) {
            self.next_board = Some(m.local() as u8);
        } else {
            self.next_board = None;
        }
        self.game_state = self.global_states.check_winner(self.next_player);
        self.next_player = self.next_player.other();
        return Ok(());
    }

    pub fn all_moves<'a>(&'a self) -> impl Iterator<Item = Move> + 'a {
        MoveIterator::from_game(&self)
    }

    pub fn game_state(&self) -> GameState {
        self.game_state
    }

    pub fn game_over(&self) -> bool {
        match self.game_state {
            GameState::InPlay => false,
            _ => true,
        }
    }

    pub fn player(&self) -> Player {
        self.next_player
    }

    pub fn board_to_play(&self) -> Option<usize> {
        self.next_board.map(|b| b as usize)
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

struct MoveIterator<'a> {
    game: &'a Game,
    board: usize,
    mask: u32,
    cell: usize,
}

impl<'a> MoveIterator<'a> {
    fn from_game(game: &'a Game) -> Self {
        let board = match game.board_to_play() {
            Some(b) => b,
            None => (0..9).find(|b| game.global_states.in_play(*b)).unwrap_or(9),
        };
        MoveIterator {
            game: game,
            cell: 0,
            mask: game.local_boards.mask(board),
            board: board,
        }
    }
}

impl<'a> Iterator for MoveIterator<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.cell >= 9 {
                if let Some(_) = self.game.board_to_play() {
                    return None;
                }
                self.board += 1;
                if !self.game.global_states.in_play(self.board) {
                    continue;
                }
                if self.board >= 9 {
                    return None;
                }
                self.cell = 0;
                self.mask = self.game.local_boards.mask(self.board);
            }

            let bit = self.mask & 1 == 0;
            let cell = self.cell;
            self.mask >>= 1;
            self.cell += 1;
            if bit {
                return Some(Move::from_coords(self.board, cell));
            }
        }
    }
}

fn codingame() {
    // game loop
    loop {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let opponent_row = parse_input!(inputs[0], i32);
        let opponent_col = parse_input!(inputs[1], i32);
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let valid_action_count = parse_input!(input_line, i32);
        for i in 0..valid_action_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let row = parse_input!(inputs[0], i32);
            let col = parse_input!(inputs[1], i32);
        }

        // Write an action using println!("message...");
        // To debug: eprintln!("Debug message...");

        println!("0 0");
    }
}

fn main() {

    codingame();
}
