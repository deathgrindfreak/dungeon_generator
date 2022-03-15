extern crate rand;

use std::collections::HashSet;

use rand::{Rng, thread_rng};

const DARK_COLOR: (u32, u32, u32) = (0, 0, 0);
const LIGHT_COLOR: (u32, u32, u32) = (199, 192, 177);
const MIN_ROOM_SIZE: usize = 5;
const MAX_ROOM_SIZE: usize = 10;

fn main() {
    let mut maze = Maze::new(110, 70);
    maze.draw_rooms(300);
    maze.print_maze();
}

struct Maze {
    width: usize,
    height: usize,
    grid_width: usize,
    grid_height: usize,
    contents: Vec<(u32, u32, u32)>,
    rooms: HashSet<Rect>,
    grid_size: usize,
}

#[derive(PartialEq, Eq, Hash)]
struct Rect((usize, usize), (usize, usize));

impl Maze {
    fn new(grid_width: usize, grid_height: usize) -> Self {
        let grid_size = 7;
        let width = grid_width * grid_size + 1;
        let height = grid_height * grid_size + 1;

        Self {
            width,
            height,
            grid_width,
            grid_height,
            contents: vec![DARK_COLOR; width * height],
            rooms: HashSet::new(),
            grid_size,
        }
    }

    fn draw_rooms(
        &mut self,
        attempts: usize
    ) {
        for _ in 0..attempts {
            let room = self.random_rectangle(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            if self.rooms.iter().all(|r| rectangles_dont_intersect(r, &room)) {
                let Rect(location, (w, h)) = room;
                self.set_grid_rectangle(location, w, h);
                self.rooms.insert(room);
            }
        }
    }

    fn random_rectangle(
        &mut self,
        min_size: usize,
        max_size: usize,
    ) -> Rect {
        let mut rnd = thread_rng();
        let (w, h) = (rnd.gen_range(min_size..=max_size), rnd.gen_range(min_size..=max_size));
        let location = (
            rnd.gen_range(0..self.grid_width - w),
            rnd.gen_range(0..self.grid_height - h)
        );
        Rect(location, (w, h))
    }

    fn set_grid_rectangle(
        &mut self,
        (x, y): (usize, usize),
        w: usize,
        h: usize
    ) {
        for i in 0..h {
            for j in 0..w {
                self.set_grid((x + j) * self.grid_size, (y + i) * self.grid_size);
            }
        }
    }

    fn set_grid(&mut self, x: usize, y: usize) {
        // Single black border
        self.set_rectangle(DARK_COLOR, (x, y), self.grid_size, self.grid_size);
        self.set_rectangle(LIGHT_COLOR, (x + 1, y + 1), self.grid_size - 1, self.grid_size - 1);
    }

    fn set_rectangle(
        &mut self,
        color: (u32, u32, u32),
        (x, y): (usize, usize),
        w: usize,
        h: usize
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

// We don't use <= here in order to allow at least one space between rooms
fn rectangles_dont_intersect(
    &Rect((x1, y1), (w1, h1)): &Rect,
    &Rect((x2, y2), (w2, h2)): &Rect
) -> bool {
    x1 + w1 < x2 || y1 + h1 < y2 || x2 + w2 < x1 || y2 + h2 < y1
}
