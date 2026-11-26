use crate::maze::{Dir, Maze, Tile, HOUSE_EXIT, HOUSE_ROW, PAC_START, WIDTH};

pub const FPS: u32 = 60;

pub const SUB: i32 = 16;

const BASE_SPEED: f32 = 9.0;
const RELEASE_DOTS: [u32; 4] = [0, 0, 15, 35];
const IDLE_RELEASE: u32 = 4 * FPS;
const RELEASE_GAP: u32 = FPS / 2;

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
            (self.x + SUB / 2).div_euclid(SUB).rem_euclid(WIDTH),
            (self.y + SUB / 2).div_euclid(SUB),
        )
    }

    fn centred(self) -> bool {
        self.x.rem_euclid(SUB) == 0 && self.y.rem_euclid(SUB) == 0
    }

    fn step(&mut self, dir: Dir) {
        let (dx, dy) = dir.delta();
        self.x = (self.x + dx).rem_euclid(WIDTH * SUB);
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GhostState {
    /// waiting in the house, bobbing up and down
    House,
    /// on the way out through the door
    Leaving,
    /// out in the maze
    Active,
}

pub struct Ghost {
    pub pos: Pos,
    pub dir: Dir,
    pub state: GhostState,
    budget: f32,
}

impl Ghost {
    fn new(index: usize) -> Self {
        let (exit_x, exit_y) = HOUSE_EXIT;
        let home_x = match index {
            2 => exit_x - 2,
            3 => exit_x + 2,
            _ => exit_x,
        };
        let home = Pos::tile_centre(home_x, HOUSE_ROW);
        let (pos, state) = if index == 0 {
            (Pos::tile_centre(exit_x, exit_y), GhostState::Active)
        } else {
            (home, GhostState::House)
        };
        Self {
            pos,
            dir: if index == 2 { Dir::Up } else { Dir::Down },
            state,
            budget: 0.0,
        }
    }

    fn update(&mut self, maze: &Maze, target: (i32, i32), speed: f32) {
        for _ in 0..steps(&mut self.budget, speed) {
            self.step(maze, target);
        }
    }

    fn step(&mut self, maze: &Maze, target: (i32, i32)) {
        let door_x = HOUSE_EXIT.0 * SUB;
        let exit_y = HOUSE_EXIT.1 * SUB;
        let house_y = HOUSE_ROW * SUB;
        match self.state {
            GhostState::House => {
                if self.pos.y <= house_y - SUB / 4 {
                    self.dir = Dir::Down;
                } else if self.pos.y >= house_y + SUB / 4 {
                    self.dir = Dir::Up;
                }
            }
            GhostState::Leaving => {
                // back to the middle row, across to the door, then straight up
                if self.pos.x != door_x {
                    self.dir = if self.pos.y < house_y {
                        Dir::Down
                    } else if self.pos.y > house_y {
                            Dir::Up
                        } else if self.pos.x < door_x {
                            Dir::Right
                        } else {
                            Dir::Left
                        };
                } else if self.pos.y > exit_y {
                    self.dir = Dir::Up;
                } else {
                    self.state = GhostState::Active;
                    self.dir = Dir::Left;
                    return;
                }
            }
            GhostState::Active => {
                if self.pos.centred() {
                    self.dir = self.pick(maze, target);
                }
            }
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
    dots_eaten: u32,
    idle: u32,
    release_wait: u32,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Self {
            maze: Maze::new(),
            pac: Pacman::new(),
            ghosts: std::array::from_fn(Ghost::new),
            score: 0,
            frame: 0,
            dots_eaten: 0,
            idle: 0,
            release_wait: 0,
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
        self.release();
        let target = self.pac.pos.tile();
        for ghost in &mut self.ghosts {
            let (x, y) = ghost.pos.tile();
            let speed = match ghost.state {
                GhostState::House | GhostState::Leaving => 0.5,
                GhostState::Active if Maze::is_tunnel(x, y) => 0.4,
                GhostState::Active => 0.75,
            };
            ghost.update(&self.maze, target, BASE_SPEED * speed);
        }
    }

    fn eat(&mut self) {
        let (x, y) = self.pac.pos.tile();
        let points = match self.maze.eat(x, y) {
            Tile::Dot => 10,
            Tile::Power => 50,
            _ => {
                self.idle += 1;
                return;
            }
        };
        self.score += points;
        self.dots_eaten += 1;
        self.idle = 0;
    }

    fn release(&mut self) {
        if self.release_wait > 0 {
            self.release_wait -= 1;
            return;
        }
        let Some(i) = self
        .ghosts
        .iter()
            .position(|g| g.state == GhostState::House)
            else {
            return;
        };
        if self.dots_eaten >= RELEASE_DOTS[i] || self.idle >= IDLE_RELEASE {
            self.ghosts[i].state = GhostState::Leaving;
            self.idle = 0;
            self.release_wait = RELEASE_GAP;
        }
    }
}
