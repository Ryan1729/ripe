#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dir {
    #[default]
    Left,
    Right,
    Up,
    Down,
}

pub type DirFlag = u8;

impl Dir {
    pub const ALL: [Dir; 4] = [
        Dir::Left,
        Dir::Right,
        Dir::Up,
        Dir::Down,
    ];

    pub const OPPOSITE: [Dir; 4] = [
        Dir::Right,
        Dir::Left,
        Dir::Down,
        Dir::Up,
    ];

    pub const FLAG: [DirFlag; 4] = [
        1 << 0,
        1 << 1,
        1 << 2,
        1 << 3,
    ];
    
    pub const fn u8(self) -> u8 {
        self as u8
    }

    pub const fn index(self) -> usize {
        self.u8() as usize
    }

    pub const fn flag(self) -> DirFlag {
        Self::FLAG[self.index()]
    }

    pub const fn opposite(self) -> Dir {
        Self::OPPOSITE[self.index()]
    }

    pub const fn moves_in_x(self) -> bool {
        self.u8() == Dir::Left.u8()
        || self.u8() == Dir::Right.u8()
    }

    pub const fn moves_in_y(self) -> bool {
        self.u8() == Dir::Up.u8()
        || self.u8() == Dir::Down.u8()
    }
}