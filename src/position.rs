#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub(crate) file: i8,
    pub(crate) rank: i8,
}

impl Position {
    pub fn new(file: i8, rank: i8) -> Position {
        Position { file, rank }
    }

    pub(crate) fn from_index(index: u8) -> Position {
        Position {
            file: (index / 8) as i8,
            rank: (index % 8) as i8,
        }
    }

    #[inline(always)]
    pub(crate) fn in_bounds(&self, index: u8, color: i8) -> bool {
        let file = (index / 8) as i8;
        let rank = (index % 8) as i8;
        file + self.file * color <= 7
            && file + self.file * color >= 0
            && rank + self.rank * color <= 7
            && rank + self.rank * color >= 0
    }

    pub(crate) fn rotate_180(mut self) -> Self {
        // rotate 180 degrees
        self.rank *= -1;
        self.file *= -1;
        self
    }

    pub(crate) fn rotate_90(mut self) -> Self {
        // rotate 90 degrees
        let file = self.file;
        self.file = -self.rank;
        self.rank = file;
        self
    }

    pub(crate) fn index(&self) -> i8 {
        self.file * 8 + self.rank
    }

    fn chebyshev_distance(&self) -> i8 {
        std::cmp::max(self.file.abs(), self.rank.abs())
    }

    pub fn is_on_path_between(&self, p1: Position, p2: Position) -> bool {
        let diff = p1 - p2;
        let diff1 = p1 - self.clone();
        let diff2 = self.clone() - p2;
        if diff.file.abs() != diff.rank.abs() && diff.file != 0 && diff.rank != 0 {
            // invalid diff, probably from knight
            false
        } else if diff.chebyshev_distance() < 2 {
            // no point between
            false
        } else if diff1.chebyshev_distance() + diff2.chebyshev_distance()
            != diff.chebyshev_distance()
        {
            // not on line
            false
        } else {
            true
        }
    }

    fn norm(mut self) -> Self {
        self.file = self.file.signum();
        self.rank = self.rank.signum();
        self
    }
}

impl std::ops::Add<u8> for Position {
    type Output = u8;

    #[inline(always)]
    fn add(self, other: u8) -> u8 {
        let file = (other / 8) as i8;
        let rank = (other % 8) as i8;
        ((file + self.file) * 8 + rank + self.rank) as u8
    }
}

impl std::ops::Sub<Position> for Position {
    type Output = Position;

    #[inline(always)]
    fn sub(mut self, other: Position) -> Self {
        self.file -= other.file;
        self.rank -= other.rank;
        self
    }
}

impl std::ops::Mul<i8> for Position {
    type Output = Position;

    fn mul(mut self, _rhs: i8) -> Position {
        self.file *= _rhs;
        self.rank *= _rhs;
        self
    }
}