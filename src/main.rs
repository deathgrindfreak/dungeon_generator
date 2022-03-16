extern crate rand;

use std::slice::Iter;
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;

const DARK_COLOR: (u32, u32, u32) = (0, 0, 0);
const LIGHT_COLOR: (u32, u32, u32) = (199, 192, 177);

fn main() {
    let mut maze = Maze::new(110, 70);
    maze.draw_rooms(400);
    maze.fill_maze();
    maze.print_maze();
}

struct Maze {
    width: usize,
    height: usize,
    grid_width: usize,
    grid_height: usize,
    contents: Vec<(u32, u32, u32)>,
    cells: Vec<bool>,
    rooms: Vec<Rect>,
    grid_size: usize,
    rnd: ThreadRng,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {N, S, E, W}

impl Direction {
    pub fn iterator() -> Iter<'static, Direction> {
        static DIRECTIONS: [Direction; 4] =
            [Direction::N, Direction::S, Direction::E, Direction::W];
        DIRECTIONS.iter()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Rect(usize, usize, usize, usize);

impl Maze {
    pub fn new(grid_width: usize, grid_height: usize) -> Self {
        let grid_size = 7;
        let width = grid_width * grid_size + 1;
        let height = grid_height * grid_size + 1;

        Self {
            width,
            height,
            grid_width,
            grid_height,
            contents: vec![DARK_COLOR; width * height],
            cells: vec![false; grid_width * grid_height],
            rooms: Vec::new(),
            grid_size,
            rnd: thread_rng(),
        }
    }

    pub fn fill_maze(&mut self) {
        self.do_fill_maze(0, 0)
    }

    fn do_fill_maze(&mut self, x: usize, y: usize) {
        if self.can_cell_be_carved(x, y) {
            self.draw_cell(x, y);

            self.neighbors(x, y)
                .iter()
                .for_each(|&(_, x, y)| self.do_fill_maze(x, y));
        }
    }

    fn can_cell_be_carved(&self, x: usize, y: usize) -> bool {
        if self.is_cell_carved(x, y) { return false; }

        let carved_neighbors: Vec<(Direction, usize, usize)> =
            self.neighbors(x, y)
                .into_iter()
                .filter(|&(_, i, j)| self.is_cell_carved(i, j))
                .collect();

        carved_neighbors.len() <= 1
    }

    fn neighbors(&self, x: usize, y: usize) -> [(Direction, usize, usize); 4] {
        [
            (Direction::N, x, if y > 0 { y-1 } else { 0 }),
            (Direction::S, x, (y+1).min(self.grid_height - 1)),
            (Direction::E, (x+1).min(self.grid_width - 1), y),
            (Direction::W, if x > 0 { x-1 } else { 0 }, y),
        ]
    }

    pub fn draw_rooms(
        &mut self,
        attempts: usize
    ) {
        for _ in 0..attempts {
            let room = self.gen_rectangle();
            if self.rooms.iter().all(|r| rectangles_dont_intersect(r, &room)) {
                self.draw_room(&room);
                self.rooms.push(room);
            }
        }
    }

    fn gen_rectangle(&mut self) -> Rect {
        let rnd = &mut self.rnd;
        let size = rnd.gen_range(2..4) * 2 + 1;
        let rectangularity = rnd.gen_range(0..1 + size / 2) * 2;

        let (w, h) = if rnd.gen_bool(0.5) {
            (size + rectangularity, size)
        } else {
            (size, size + rectangularity)
        };

        let x = rnd.gen_range(0..(self.grid_width - w) / 2) * 2 + 1;
        let y = rnd.gen_range(0..(self.grid_height - h) / 2) * 2 + 1;

        Rect(x, y, w, h)
    }

    fn draw_room(
        &mut self,
        &Rect(x, y, w, h): &Rect,
    ) {
        for i in 0..h {
            for j in 0..w {
                self.draw_cell(x + j, y + i);
            }
        }
    }

    fn is_cell_carved(&self, x: usize, y: usize) -> bool {
        self.cells[y * self.grid_width + x]
    }

    // Graphics primitives

    fn draw_cell(&mut self, x: usize, y: usize) {
        // Set this here so that we're using grid coordinates
        self.cells[y * self.grid_width + x] = true;

        // Convert to pixel coordinates
        let (x, y) = (x * self.grid_size, y * self.grid_size);

        // Make room for a border the color of DARK_COLOR
        self.draw_rectangle(LIGHT_COLOR, &Rect(x + 1, y + 1, self.grid_size - 1, self.grid_size - 1));
    }

    fn draw_rectangle(
        &mut self,
        color: (u32, u32, u32),
        &Rect(x, y, w, h): &Rect,
    ) {
        for j in y..(y+h) {
            for i in x..(x+w) {
                self.contents[j * self.width + i] = color;
            }
        }
    }

    fn print_maze(&self) {
        println!("P3\n{} {}\n255\n", self.width, self.height);

        for j in 0..self.height {
            for i in 0..self.width {
                let (r, g, b) = self.contents[j * self.width + i];
                println!("{} {} {}", r, g, b);
            }
        }
    }
}

// Use < instead of <= here in order to allow at least one space between rooms
fn rectangles_dont_intersect(
    &Rect(x1, y1, w1, h1): &Rect,
    &Rect(x2, y2, w2, h2): &Rect
) -> bool {
    x1 + w1 < x2 || y1 + h1 < y2 || x2 + w2 < x1 || y2 + h2 < y1
}
