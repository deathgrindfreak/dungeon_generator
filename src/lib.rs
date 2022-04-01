use std::ops;
use std::slice::Iter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec2D(pub i32, pub i32);

impl ops::Mul<i32> for Vec2D {
    type Output = Vec2D;

    fn mul(self, t: i32) -> Vec2D {
        let Vec2D(x, y) = self;
        Vec2D(x * t, y * t)
    }
}

impl ops::Mul<Vec2D> for i32 {
    type Output = Vec2D;

    fn mul(self, Vec2D(x, y): Vec2D) -> Vec2D {
        Vec2D(x * self, y * self)
    }
}

impl ops::Add<Vec2D> for Vec2D {
    type Output = Vec2D;

    fn add(self, Vec2D(x2, y2): Vec2D) -> Vec2D {
        let Vec2D(x1, y1) = self;
        Vec2D(x1 + x2, y1 + y2)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {N, S, E, W}

impl Direction {
    pub fn iterator() -> Iter<'static, Direction> {
        static DIRECTIONS: [Direction; 4] =
            [Direction::N, Direction::S, Direction::E, Direction::W];
        DIRECTIONS.iter()
    }

    pub fn dir(&self) -> Vec2D {
        match self {
            Direction::N => Vec2D(0, -1),
            Direction::S => Vec2D(0, 1),
            Direction::E => Vec2D(1, 0),
            Direction::W => Vec2D(-1, 0),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Rect(pub i32, pub i32, pub i32, pub i32);

impl Rect {
    // Use < instead of <= here in order to allow at least one space between rooms
    pub fn is_outside_of(&self, &Rect(x2, y2, w2, h2): &Rect) -> bool {
        let &Rect(x1, y1, w1, h1) = self;
        x1 + w1 < x2 || y1 + h1 < y2 || x2 + w2 < x1 || y2 + h2 < y1
    }

    pub fn contains(&self, Vec2D(i, j): Vec2D) -> bool {
        let &Rect(x, y, w, h) = self;
        x as i32 <= i && i < (x + w) as i32
            && y as i32 <= j && j < (y + h) as i32
    }

    pub fn x(&self) -> i32 {
        self.0
    }

    pub fn y(&self) -> i32 {
        self.1
    }

    pub fn width(&self) -> i32 {
        self.2
    }

    pub fn height(&self) -> i32 {
        self.3
    }
}
