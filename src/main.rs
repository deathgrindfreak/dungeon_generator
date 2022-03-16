extern crate rand;

use rand::{Rng, thread_rng};
use rand::rngs::{ThreadRng};

const DARK_COLOR: (u32, u32, u32) = (0, 0, 0);
const LIGHT_COLOR: (u32, u32, u32) = (199, 192, 177);

fn main() {
    let mut maze = Maze::new(110, 70);
    maze.draw_rooms(400);
    maze.print_maze();
}

struct Maze {
    width: usize,
    height: usize,
    grid_width: usize,
    grid_height: usize,
    contents: Vec<(u32, u32, u32)>,
    rooms: Vec<Rect>,
    grid_size: usize,
    rnd: ThreadRng,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Rect(usize, usize, usize, usize);

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
            rooms: Vec::new(),
            grid_size,
            rnd: thread_rng(),
        }
    }

    fn draw_rooms(
        &mut self,
        attempts: usize
    ) {
        for _ in 0..attempts {
            let room = self.random_rectangle();
            if self.rooms.iter().all(|r| rectangles_dont_intersect(r, &room)) {
                self.set_grid_rectangle(&room);
                self.rooms.push(room);
            }
        }
    }

    fn random_rectangle(&mut self) -> Rect {
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

    fn set_grid_rectangle(
        &mut self,
        &Rect(x, y, w, h): &Rect,
    ) {
        for i in 0..h {
            for j in 0..w {
                self.set_grid((x + j) * self.grid_size, (y + i) * self.grid_size);
            }
        }
    }

    fn set_grid(&mut self, x: usize, y: usize) {
        // Make room for a border the color of DARK_COLOR
        self.set_rectangle(LIGHT_COLOR, &Rect(x + 1, y + 1, self.grid_size - 1, self.grid_size - 1));
    }

    fn set_rectangle(
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
