use crate::maze::{Dir, Maze, Tile, PAC_START};

pub const FPS: u32 = 60;

pub const SUB: i32 = 16;

const BASE_SPEED: f32 = 9.0;
const GHOST_STARTS: [(i32, i32); 4] = [(9, 9), (11, 9), (9, 10), (11, 10)];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn tile_centre(x: i32, y: i32) -> Self {
        Self {
            x: x * SUB,
            y: y * SUB,
        }
    }

    pub fn tile(self) -> (i32, i32) {
        (
            (self.x + SUB / 2).div_euclid(SUB),
            (self.y + SUB / 2).div_euclid(SUB),
        )
    }

    fn centred(self) -> bool {
        self.x.rem_euclid(SUB) == 0 && self.y.rem_euclid(SUB) == 0
    }

    fn step(&mut self, dir: Dir) {
        let (dx, dy) = dir.delta();
        self.x += dx;
        self.y += dy;
    }
}

fn steps(budget: &mut f32, tiles_per_sec: f32) -> u32 {
    *budget += tiles_per_sec * SUB as f32 / FPS as f32;
    let whole = budget.floor();
    *budget -= whole;
    whole as u32
}

fn dist2(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).pow(2) + (a.1 - b.1).pow(2)
}

pub struct Pacman {
    pub pos: Pos,
    pub dir: Dir,

    want: Dir,
    budget: f32,
    /// sub steps walked, drives the mouth
    chomp: u32,
}

impl Pacman {
    fn new() -> Self {
        Self {
            pos: Pos::tile_centre(PAC_START.0, PAC_START.1),
            dir: Dir::Left,
            want: Dir::Left,
            budget: 0.0,
            chomp: 0,
        }
    }

    pub fn update(&mut self, maze: &Maze, speed: f32) {
        for _ in 0..steps(&mut self.budget, speed) {
            // Reversing is allowed anywhere, not just at tile centres.
            if self.want == self.dir.opposite() {
                self.dir = self.want;
            }

            if self.pos.centred() {
                let (x, y) = self.pos.tile();

                // Take the queued turn if that way is open.
                let (wx, wy) = self.want.delta();
                if !maze.blocked(x + wx, y + wy) {
                    self.dir = self.want;
                }

                // Stop at walls.
                let (dx, dy) = self.dir.delta();
                if maze.blocked(x + dx, y + dy) {
                    break;
                }
            }

            self.pos.step(self.dir);
            self.chomp = self.chomp.wrapping_add(1);
        }
    }

    pub fn mouth(&self) -> f32 {
        let phase = self.chomp / 2 % 8;
        let open = if phase < 4 { phase } else { 8 - phase };
        open as f32 / 4.0
    }
}

pub struct Ghost {
    pub pos: Pos,
    pub dir: Dir,
    budget: f32,
}

impl Ghost {
    fn new(index: usize) -> Self {
        let (x, y) = GHOST_STARTS[index];
        Self {
            pos: Pos::tile_centre(x, y),
            dir: Dir::Left,
            budget: 0.0,
        }
    }

    fn update(&mut self, maze: &Maze, target: (i32, i32), speed: f32) {
        for _ in 0..steps(&mut self.budget, speed) {
            if self.pos.centred() {
                self.dir = self.pick(maze, target);
            }
            self.pos.step(self.dir);
        }
    }

    /// at every tile take the open way, other than straight back, that ends up
    /// closest to the target. ties go Up, Left, Down, Right
    fn pick(&self, maze: &Maze, target: (i32, i32)) -> Dir {
        let (x, y) = self.pos.tile();
        Dir::ALL
            .into_iter()
            .filter(|&d| d != self.dir.opposite())
            .filter(|&d| {
                let (dx, dy) = d.delta();
                !maze.blocked(x + dx, y + dy)
            })
            .min_by_key(|&d| {
                let (dx, dy) = d.delta();
                dist2((x + dx, y + dy), target)
            })
            .unwrap_or(self.dir.opposite())
    }
}

pub struct Game {
    pub maze: Maze,
    pub pac: Pacman,
    pub ghosts: [Ghost; 4],
    pub score: u32,
    /// counts every update, for animations
    pub frame: u64,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Self {
            maze: Maze::new(),
            pac: Pacman::new(),
            ghosts: std::array::from_fn(Ghost::new),
            score: 0,
            frame: 0,
        };
        game.eat();
        game
    }

    pub fn steer(&mut self, dir: Dir) {
        self.pac.want = dir;
    }

    pub fn update(&mut self) {
        self.frame += 1;
        self.pac.update(&self.maze, BASE_SPEED * 0.8);
        self.eat();
        let target = self.pac.pos.tile();
        for ghost in &mut self.ghosts {
            ghost.update(&self.maze, target, BASE_SPEED * 0.75);
        }
    }

    fn eat(&mut self) {
        let (x, y) = self.pac.pos.tile();
        match self.maze.eat(x, y) {
            Tile::Dot => self.score += 10,
            Tile::Power => self.score += 50,
            _ => {}
        }
    }
}
