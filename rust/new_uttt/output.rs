mod game {
    mod masks {
        #[inline]
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
        #[inline]
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
    }
    pub struct Move {
        bits: usize,
    }
    impl Move {
        #[inline]
        pub fn from_coords(global: usize, local: usize) -> Self {
            Self { bits: (global << 9 | local) }
        }
        #[inline]
        pub fn global(self) -> usize {
            self.bits >> 9
        }
        #[inline]
        pub fn local(self) -> usize {
            self.bits & masks::BOARD_MASK
        }
        #[inline]
        pub fn print(&self) {
            let row = self.global() / 3 * 3 + masks::local_to_global(self.local()) / 3;
            let col = self.global() % 3 * 3 + masks::local_to_global(self.local()) % 3;
            println!("{} {}", row, col);
        }
    }
}
mod mcts {
    use std::cell::RefCell;
    use std::rc::Weak;
    pub trait Action: Eq + Clone {}
    pub struct Node<A>
    where
        A: Action,
    {
        action: A,
        parent: Option<Weak<Node<A>>>,
        children: RefCell<Vec<Node<A>>>,
        wins: f64,
        visits: f64,
    }
}
pub use mcts::Node;
fn main() {
    let root = Node::new();
}

