extern crate clap;
extern crate dungeon_generator;
extern crate rand;

use std::collections::HashSet;

use clap::Parser;
use rand::prelude::SliceRandom;
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
use dungeon_generator::{PPM, Color, Direction, Vec2D, Rect};

const CELL_SIZE: i32 = 7;
const DARK_COLOR: Color = Color(0, 0, 0);
const LIGHT_COLOR: Color = Color(199, 192, 177);
const DOOR_COLOR: Color = Color(230, 100, 0);

const WINDING_PERCENT: i32 = 0;

/// Generates a random dungeon
#[derive(Parser, Debug)]
struct Args {
    /// Generates a PPM stream (intended for created video with ffmpeg, etc ...)
    #[clap(long)]
    animate: bool,

    /// The width of the dungeon in "grids" (This number must be odd)
    #[clap(short, long, default_value_t = 121)]
    width: i32,

    /// The height of the dungeon in "grids" (This number must be odd)
    #[clap(short, long, default_value_t = 91)]
    height: i32,

    /// The number attempts the program should use to place rooms
    #[clap(short, long, default_value_t = 200)]
    attempts: i32,
}

fn main() {
    let args = Args::parse();
    Dungeon::new(args).generate();
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tile {
    Floor,
    Wall,
    Door,
}

struct Dungeon {
    image: PPM,

    bounds: Rect,

    cells: Vec<Tile>,
    rooms: Vec<Rect>,

    attempts: i32,
    animate: bool,
    rnd: ThreadRng,
}

impl Dungeon {
    pub fn new(args: Args) -> Self {
        let Args {
            width: grid_width,
            height: grid_height,
            animate,
            attempts,
        } = args;

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

            attempts,
            animate,
            rnd: thread_rng(),
        }
    }

    pub fn generate(&mut self) {
        self.draw_rooms();
        self.fill_maze();
        self.find_connectors();
        self.remove_dead_ends();
        self.print_dungeon();
    }

    fn remove_dead_ends(&mut self) {
        let room_cells: HashSet<Vec2D> = self.rooms
                                             .iter()
                                             .flat_map(|&r| r.into_iter())
                                             .collect();

        let mut dead_ends: Vec<Vec2D> =
            self.cells
                .iter()
                .enumerate()
                .filter_map(|(i, t)| {
                    match t {
                        Tile::Floor | Tile::Door => {
                            let x = i as i32 % self.bounds.width();
                            let y = (i as i32 - x) / self.bounds.width();
                            Some(Vec2D(x, y))
                        },
                        Tile::Wall => None,
                    }
                })
                .into_iter()
                .collect::<HashSet<Vec2D>>()
                .difference(&room_cells)
                .filter(|&&cell| {
                    Direction::iterator().map(|&d| {
                        if self.get_tile(cell + d.dir()) == Tile::Floor { 1 } else { 0 }
                    }).sum::<i32>() == 1
                })
                .map(|&d| d)
                .collect();

        while !dead_ends.is_empty() {
            let cell = dead_ends.pop().unwrap();

            // If we have a door as a neighbor, just skip filling this in
            if Direction::iterator().any(|&d| self.get_tile(cell + d.dir()) == Tile::Door) {
                continue;
            }

            self.fill_cell(cell);

            if self.animate {
                self.print_dungeon();
            }

            if let Some(d) = Direction::iterator().find(|&d| self.get_tile(cell + d.dir()) == Tile::Floor) {
                let next_cell = cell + d.dir();

                let number_of_floors = Direction::iterator().map(|&d| {
                    if self.get_tile(next_cell + d.dir()) == Tile::Floor { 1 } else { 0 }
                }).sum::<i32>();

                // If we're still in a single, long corridor keep removing
                if number_of_floors == 1 {
                    dead_ends.push(next_cell);
                }
            }
        }
    }

    fn find_connectors(&mut self) {
        let connectors: Vec<(Rect, Vec<Vec2D>)> = self.rooms.iter().map(|&room| {
            let connected_rooms: Vec<Vec2D> = room.into_iter().filter_map(|e| {
                is_edge_and_not_corner(&room, &e)
                    .filter(|&(v, d)| {
                        self.bounds.contains(v + 3 * d.dir())
                            && self.get_tile(v + 2 * d.dir()) == Tile::Floor
                    })
                    .map(|(v, d)| v + d.dir())
            }).collect();

            (room, connected_rooms)
        }).collect();

        let mut all_connectors: HashSet<Vec2D> = connectors
            .iter()
            .flat_map(|(_, r)| r)
            .map(|&d| d)
            .collect();

        for (room, cs) in connectors.iter() {
            let available_connectors: Vec<Vec2D> = cs
                .iter()
                .filter(|&c| all_connectors.contains(c))
                .map(|&c| c)
                .collect();

            let &door = available_connectors.choose(&mut self.rnd).unwrap();
            self.carve_door(door);
            self.draw_room(room, Some(LIGHT_COLOR));

            for &c in available_connectors.iter() {
                // Randomly open a passage that's to be culled
                if c != door && self.rnd.gen_bool(1.0 / 50.0) {
                    self.carve_door(c);
                }

                all_connectors.remove(&c);
            }

            if self.animate {
                self.print_dungeon();
            }
        }

        fn is_edge_and_not_corner(
            &Rect(x, y, w, h): &Rect,
            &v @ Vec2D(i, j): &Vec2D
        ) -> Option<(Vec2D, Direction)> {
            if x < i && i < x + w - 1 && j == y {
                Some((v, Direction::N))
            } else if x < i && i < x + w - 1 && j == y + h - 1 {
                Some((v, Direction::S))
            } else if i == x && y < j && j < y + h - 1 {
                Some((v, Direction::W))
            } else if i == x + w - 1 && y < j && j < y + h - 1 {
                Some((v, Direction::E))
            } else {
                None
            }
        }
    }

    fn fill_maze(&mut self) {
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
        self.carve_floor(start);

        if self.animate {
            self.print_dungeon();
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

                self.carve_floor(cell + d.dir());
                self.carve_floor(cell + d.dir() * 2);

                cells.push(cell + d.dir() * 2);

                if self.animate {
                    self.print_dungeon();
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

    fn draw_rooms(&mut self) {
        for _ in 0..self.attempts {
            let room = self.gen_rectangle();
            if self.rooms.iter().all(|r| r.distance_to(&room).unwrap_or(0) > 0) {
                self.draw_room(&room, None);
                self.rooms.push(room);

                if self.animate {
                    self.print_dungeon();
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
        default_color: Option<Color>
    ) {
        let color = default_color.unwrap_or_else(|| {
            // Give it a random color, that's kind of light
            let rnd = &mut self.rnd;
            let r = rnd.gen_range(125..=255);
            let g = rnd.gen_range(125..=255);
            let b = rnd.gen_range(125..=255);
            Color(r, g, b)
        });

        for point in rect.into_iter() {
            self.carve_cell(point, color, Tile::Floor);
        }
    }

    // Graphics primitives

    fn set_cell(&mut self, Vec2D(x, y): Vec2D, value: Tile) {
        self.cells[(y * self.bounds.width() + x) as usize] = value;
    }

    fn fill_cell(&mut self, v: Vec2D) {
        self.carve_cell(v, DARK_COLOR, Tile::Wall);
    }

    fn carve_floor(&mut self, v: Vec2D) {
        self.carve_cell(v, LIGHT_COLOR, Tile::Floor);
    }

    fn carve_door(&mut self, v: Vec2D) {
        self.carve_cell(v, DOOR_COLOR, Tile::Door);
    }

    fn carve_cell(
        &mut self,
        v @ Vec2D(x, y): Vec2D,
        color: Color,
        tile: Tile,
    ) {
        self.set_cell(v, tile);

        // Convert to pixel coordinates
        let (x, y) = (x * CELL_SIZE, y * CELL_SIZE);

        // Create a border slightly darker than the color
        self.image.draw_rectangle(&Rect(x, y, CELL_SIZE, CELL_SIZE), color.darker(0.70));

        // Allow room for border
        self.image.draw_rectangle(&Rect(x + 1, y + 1, CELL_SIZE - 1, CELL_SIZE - 1), color);
    }

    fn print_dungeon(&self) {
        self.image.print();
    }
}
