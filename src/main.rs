extern crate clap;
extern crate maze;
extern crate rand;

use clap::Parser;
use rand::prelude::SliceRandom;
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
use maze::{PPM, Color, Direction, Vec2D, Rect};

const CELL_SIZE: i32 = 7;
const DARK_COLOR: Color = (0, 0, 0);
const LIGHT_COLOR: Color = (199, 192, 177);

const WINDING_PERCENT: i32 = 0;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long)]
    animate: bool,
}

fn main() {
    let args = Args::parse();
    let mut maze = Maze::new(115, 83, args.animate);
    maze.draw_rooms(400);
    maze.fill_maze();
    maze.print_maze();
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tile {
    Floor,
    Wall
}

struct Maze {
    image: PPM,

    bounds: Rect,

    cells: Vec<Tile>,
    rooms: Vec<Rect>,

    animate: bool,
    rnd: ThreadRng,
}

impl Maze {
    pub fn new(grid_width: i32, grid_height: i32, animate: bool) -> Self {
        if grid_width % 2 == 0 || grid_height % 2 == 0 {
            panic!("Grid must be odd-sized!");
        }

        Self {
            image: PPM::new(
                grid_width * CELL_SIZE + 1,
                grid_height * CELL_SIZE + 1,
                DARK_COLOR,
            ),

            bounds: Rect(0, 0, grid_width, grid_height),

            cells: vec![Tile::Wall; (grid_width * grid_height) as usize],
            rooms: Vec::new(),

            animate,
            rnd: thread_rng(),
        }
    }

    pub fn fill_maze(&mut self) {
        for y in (1..self.bounds.height()).step_by(2) {
            for x in (1..self.bounds.width()).step_by(2) {
                let cell = Vec2D(x, y);
                if self.get_tile(cell) == Tile::Wall {
                    self.do_fill_maze(cell);
                }
            }
        }
    }

    fn do_fill_maze(&mut self, start: Vec2D) {
        let mut cells = Vec::new();
        let mut last_dir: Option<Direction> = None;

        cells.push(start);
        self.carve(start);

        if self.animate {
            self.print_maze();
        }

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

                if self.animate {
                    self.print_maze();
                }
            }
        }
    }

    fn can_cell_be_carved(&self, &pos: &Vec2D, &d: &Direction) -> bool {
        self.bounds.contains(pos + d.dir() * 3)
            && self.get_tile(pos + d.dir() * 2) == Tile::Wall
    }

    fn get_tile(&self, Vec2D(x, y): Vec2D) -> Tile {
        self.cells[(y * self.bounds.width() + x) as usize]
    }

    pub fn draw_rooms(
        &mut self,
        attempts: usize
    ) {
        for _ in 0..attempts {
            let room = self.gen_rectangle();
            if self.rooms.iter().all(|r| r.distance_to(&room).unwrap_or(0) > 0) {
                self.draw_room(&room);
                self.rooms.push(room);

                if self.animate {
                    self.print_maze();
                }
            }
        }
    }

    fn gen_rectangle(&mut self) -> Rect {
        let rnd = &mut self.rnd;
        let size = rnd.gen_range(2..4) * 2 + 1;
        let rectangularity = rnd.gen_range(0..(1 + size / 2)) * 2;

        let (width, height) = if rnd.gen_bool(0.5) {
            (size + rectangularity, size)
        } else {
            (size, size + rectangularity)
        };

        let x = rnd.gen_range(0..(self.bounds.width() - width) / 2) * 2 + 1;
        let y = rnd.gen_range(0..(self.bounds.height() - height) / 2) * 2 + 1;

        Rect(x, y, width, height)
    }

    fn draw_room(
        &mut self,
        &rect: &Rect,
    ) {
        let rnd = &mut self.rnd;
        let r = rnd.gen_range(125..=255);
        let g = rnd.gen_range(125..=255);
        let b = rnd.gen_range(125..=255);

        for point in rect.into_iter() {
            self.carve_cell(point, (r, g, b));
        }
    }

    // Graphics primitives

    fn set_cell(&mut self, Vec2D(x, y): Vec2D, value: Tile) {
        self.cells[(y * self.bounds.width() + x) as usize] = value;
    }

    fn carve(&mut self, v: Vec2D) {
        self.carve_cell(v, LIGHT_COLOR);
    }

    fn carve_cell(&mut self, Vec2D(x, y): Vec2D, color: Color) {
        self.set_cell(Vec2D(x, y), Tile::Floor);

        // Convert to pixel coordinates
        let (x, y) = (x * CELL_SIZE, y * CELL_SIZE);

        // Make room for a border the color of DARK_COLOR
        self.image.draw_rectangle(&Rect(x + 1, y + 1, CELL_SIZE - 1, CELL_SIZE - 1), color);
    }

    fn print_maze(&self) {
        self.image.print();
    }
}
