extern crate maze;
extern crate rand;

use rand::prelude::SliceRandom;
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
use maze::{Direction, Vec2D, Rect};

const CELL_SIZE: i32 = 7;
const DARK_COLOR: (u32, u32, u32) = (0, 0, 0);
const LIGHT_COLOR: (u32, u32, u32) = (199, 192, 177);

const WINDING_PERCENT: u32 = 80;

fn main() {
    let mut maze = Maze::new(110, 70);
    maze.draw_rooms(400);
    maze.fill_maze();
    maze.print_maze();
}

struct Maze {
    width: i32,
    height: i32,

    bounds: Rect,

    contents: Vec<(u32, u32, u32)>,
    cells: Vec<bool>,
    rooms: Vec<Rect>,

    rnd: ThreadRng,
}

impl Maze {
    pub fn new(grid_width: i32, grid_height: i32) -> Self {
        let width = grid_width * CELL_SIZE + 1;
        let height = grid_height * CELL_SIZE + 1;

        Self {
            width,
            height,

            bounds: Rect(0, 0, grid_width, grid_height),

            contents: vec![DARK_COLOR; (width * height) as usize],
            cells: vec![false; (grid_width * grid_height) as usize],
            rooms: Vec::new(),

            rnd: thread_rng(),
        }
    }

    pub fn fill_maze(&mut self) {
        self.do_fill_maze(Vec2D(0, 0))
    }

    fn do_fill_maze(&mut self, start: Vec2D) {
        let mut cells = Vec::new();
        let mut last_dir: Option<Direction> = None;

        cells.push(start);
        self.carve(start);

        while !cells.is_empty() {
            let &cell = cells.last().unwrap();

            let unmade_direction: Vec<Direction> = Direction::iterator().filter_map(|&d| {
                if self.can_cell_be_carved(&cell, &d) { Some(d) } else { None }
            }).collect();

            if unmade_direction.is_empty() {
                let _ = cells.pop();
                last_dir = None;
            } else {
                last_dir = if last_dir.is_some()
                    && unmade_direction.contains(&last_dir.unwrap())
                    && self.rnd.gen_range(0..100) > WINDING_PERCENT {
                        last_dir
                    } else {
                        unmade_direction.choose(&mut self.rnd).map(|&d| d)
                    };

                let d = last_dir.unwrap();

                self.carve(cell + d.dir());
                self.carve(cell + d.dir() * 2);

                cells.push(cell + d.dir() * 2);
            }
        }
    }

    fn can_cell_be_carved(&self, &pos: &Vec2D, &d: &Direction) -> bool {
        self.bounds.contains(pos + d.dir() * 3) && !self.is_cell_carved(pos + d.dir() * 2)
    }

    fn is_cell_carved(&self, Vec2D(x, y): Vec2D) -> bool {
        self.cells[(y * self.bounds.width() + x) as usize]
    }

    pub fn draw_rooms(
        &mut self,
        attempts: usize
    ) {
        for _ in 0..attempts {
            let room = self.gen_rectangle();
            if self.rooms.iter().all(|r| r.is_outside_of(&room)) {
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

        let x = rnd.gen_range(0..(self.bounds.width() - w) / 2) * 2 + 1;
        let y = rnd.gen_range(0..(self.bounds.height() - h) / 2) * 2 + 1;

        Rect(x, y, w, h)
    }

    fn draw_room(
        &mut self,
        &Rect(x, y, w, h): &Rect,
    ) {
        // Leave a border around the rectangle so that the maze doesn't touch it
        for i in 1..h-1 {
            for j in 1..w-1 {
                self.carve(Vec2D((x + j) as i32, (y + i) as i32));
            }
        }
    }

    // Graphics primitives

    fn set_cell(&mut self, Vec2D(x, y): Vec2D, value: bool) {
        self.cells[(y * self.bounds.width() + x) as usize] = value;
    }

    fn carve(&mut self, Vec2D(x, y): Vec2D) {
        self.set_cell(Vec2D(x, y), true);

        // Convert to pixel coordinates
        let (x, y) = (x * CELL_SIZE, y * CELL_SIZE);

        // Make room for a border the color of DARK_COLOR
        self.draw_rectangle(LIGHT_COLOR, &Rect(x + 1, y + 1, CELL_SIZE - 1, CELL_SIZE - 1));
    }

    fn draw_rectangle(
        &mut self,
        color: (u32, u32, u32),
        &Rect(x, y, w, h): &Rect,
    ) {
        for j in y..(y+h) {
            for i in x..(x+w) {
                self.contents[(j * self.width + i) as usize] = color;
            }
        }
    }

    fn print_maze(&self) {
        println!("P3\n{} {}\n255\n", self.width, self.height);

        for j in 0..self.height {
            for i in 0..self.width {
                let (r, g, b) = self.contents[(j * self.width + i) as usize];
                println!("{} {} {}", r, g, b);
            }
        }
    }
}
