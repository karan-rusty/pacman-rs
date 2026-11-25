pub const MAZE: [&str; 18] = [
    "#####################",
    "#.........#.........#",
    "#.###.###.#.###.###.#",
    "#o###.###.#.###.###o#",
    "#...................#",
    "#.###.#.#####.#.###.#",
    "#.....#...#...#.....#",
    "#####.###.#.###.#####",
    "#####.#       #.#####",
    "#####.#       #.#####",
    "#####.#       #.#####",
    "#####.#.#####.#.#####",
    "#.........#.........#",
    "#.###.###.#.###.###.#",
    "#o..#.....#.....#..o#",
    "###.#.#.#####.#.#.###",
    "#.....#...#...#.....#",
    "#####################",
];

pub const WIDTH: i32 = 21;
pub const HEIGHT: i32 = 18;

pub const PAC_START: (i32, i32) = (11, 14);

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
    Empty,
    Dot,
    Power,
}

#[derive(Clone)]
pub struct Maze {
    tiles: Vec<Tile>,
}

impl Maze {
    pub fn new() -> Self {
        let tiles: Vec<Tile> = MAZE
            .iter()
            .flat_map(|row| row.chars())
            .map(|c| match c {
                '#' => Tile::Wall,
                '.' => Tile::Dot,
                'o' => Tile::Power,
                _ => Tile::Empty,
            })
            .collect();
        Self { tiles }
    }

    pub fn get(&self, x: i32, y: i32) -> Tile {
        let tile = self.get(x, y);
        if matches!(tile, Tile::Dot | Tile::power) {
            self.tiles[(y * WIDTH + x) as usize] = Tile::Empty;
        }
        tile
    }
}

impl Default for Maze {
    fn default() -> Self {
        Self::new()
    }
}
