pub const MAZE: [&str; 20] = [
    "#########################",
    "#...........#...........#",
    "#.####.####.#.####.####.#",
    "#o####.####.#.####.####o#",
    "#.......................#",
    "#.####.#.#######.#.####.#",
    "#......#....#....#......#",
    "######.####.#.####.######",
    "######.#         #.######",
    "######.# ###-### #.######",
    "      .  #     #  .      ",
    "######.# ####### #.######",
    "######.#         #.######",
    "######.#.#######.#.######",
    "#...........#...........#",
    "#.####.####.#.####.####.#",
    "#o..#...............#..o#",
    "###.#.##.#######.##.#.###",
    "#.......................#",
    "#########################",
]

pub const WIDTH: i32 = 25;
pub const HEIGHT: i32 = 20;

pub const PAC_START: (i32, i32) = (12, 16);
pub const HOUSE_EXIT: (i32, i32) = (12, 8);
pub const HOUSE_ROW: i32 = 10;
pub const TUNNEL_ROW: i32 = 10;
pub const MESSAGE_ROW: i32 = 12;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Up,
    Left,
    Down,
    Right,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::Up, Dir::Left, Dir::Down, Dir::Right];

    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Left => (-1, 0),
            Dir::Down => (0, 1),
            Dir::Right => (1, 0),
        }
    }

    pub fn opposite(self) -> Dir {
        match self {
            Dir::Up => Dir::Down,
            Dir::Left => Dir::Right,
            Dir::Down => Dir::Up,
            Dir::Right => Dir::Left,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Wall,
    Door,
    Empty,
    Dot,
    Power,
}

#[derive(Clone)]
pub struct Maze {
    tiles: Vec<Tile>,
    dots: usize,
}

impl Maze {
    pub fn new() -> Self {
        let tiles: Vec<Tile> = MAZE
            .iter()
            .flat_map(|row| row.chars())
            .map(|c| match c {
                '#' => Tile::Wall,
                '-' => Tile::Door,
                '.' => Tile::Dot,
                'o' => Tile::Power,
                _ => Tile::Empty,
            })
            .collect();
        let dots = tiles
        .iter()
            .filter(|t| matches!(t, Tile::Dot | Tile::Power))
            .count();
        Self { tiles, dots}
    }

    pub fn get(&self, x: i32, y: i32) -> Tile {
        if !(0..HEIGHT).contains(&y) {
            return Tile::Wall;
        }
        self.tiles[(y * WIDTH + x.rem_euclid(WIDTH)) as usize]
    }

    pub fn blocked(&self, x: i32, y: i32) -> bool {
        matches!(self.get(x, y), Tile::Wall | Tile::Door)
    }

    pub fn is_tunnel(x: i32, y: i32) -> bool {
        y == TUNNEL_ROW && !(6..WIDTH - 6).contains(&x)
    }

    pub fn eat(&mut self, x: i32, y: i32) -> Tile {
        let tile = self.get(x ,y);
        if matches!(tile, Tile::Dot | Tile::Power) {
            self.tiles[(y * WIDTH + x.rem_euclid(WIDTH)) as usize] = Tile::Empty;
            self.dots -= 1;
        }
        tile
    }

    pub fn dots_left(&self) -> usize {
        self.dots
    }
}

impl Default for Maze {
    fn default() -> Self {
        Self::new()
    }
}
