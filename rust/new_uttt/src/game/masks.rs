//#[inline]
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

//#[inline]
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