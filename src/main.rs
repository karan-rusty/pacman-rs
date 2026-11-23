mod maze;

use std::{io, time::{Duration,Instant}};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    widgets::Paragraph,
    Frame,
    Terminal,
};

use gfx::{Canvas, Rgb};
use maze::Dir;

struct Board {
    tiles: Vec<Vec<char>>,
    pellets: usize,
}

impl Board {
    fn new() -> Self {
        let tiles: Vec<Vec<char>> = maze::MAZE.iter().map(|row| row.chars().collect()).collect();
        let pellets = tiles
            .iter()
            .flatten()
            .filter(|&&tile| tile == '.' || tile == 'o')
            .count();
        Self { tiles, pellets }
    }

    fn width(&self) -> i32 {
        self.tiles[0].len() as i32
    }

    fn height(&self) -> i32 {
        self.tiles.len() as i32
    }

    fn tile(&self, x: i32, y: i32) -> char {
        if x < 0 || y < 0 || x >= self.width() || y >= self.height(){
            return '#';
        }
        self.tiles[y as usize][x as usize]
    }

    fn is_wall(&self, x: i32, y: i32) -> bool {
        self.tile(x, y) == '#'
    }

    fn eat(&mut self, x: i32, y: i32) -> u32 {
        let points = match self.tile(x, y) {
            '.' => 10,
            'o' => 50,
            _ => return 0,
        };
        self.tiles[y as usize][x as usize] = ' ';
        self.pellets -= 1;
        points
    }
}

struct Pacman {
    x: i32,
    y: i32,
    dir: (i32, i32),
}

impl Pacman {
    fn step(&mut self, board: &mut Board, dir: Dir) -> u32 {
        self.facing = dir;
        let (dx, dy) = dir.delta();
        if board.is_wall(self.x + dx, self.y + dy) {
            return 0;
        }
        self.x += dx;
        self.y += dy;
        self.steps += 1;
        board.eat(self.x, self.y)
    }
}

struct Ghost {
    x: i32,
    y: i32,
    color: Rgb,
    looking: Dir,
}

impl Ghost {
    fn new(x: i32, y: i32, color: Rgb) -> Self {
        Self {
            x,
            y,
            color,
            looking: Dir::Left,
        }
    }

    fn chase(&mut self, pac: &Pacman, board: &Board) {
        let dx = pac.x - self.x;
        let dy = pac.y - self.y;
        let across = if dx < 0 { Dir::Left } else { Dir::Right };
        let down = if dy < 0 { Dir::Up } else { Dir::Down }
        let moves = if dx.abs() >= dy.abs() {
            [across, down, down.opposite(), across.opposite()]
        } else {
            [down, across, across.opposite(), down.opposite()]
        };

        for dir in moves {
            let (mx, my) = dir.delta();
            if !board.is_wall(self.x + mx, self.y + my) {
                self.x += mx;
                self.y += my;
                self.looking = dir;
                return;
            }
        }

    }
}

struct Scene {
    tile: i32,
    walls: Canvas,
    canvas: Canvas,
}

impl Scene {
    fn new() -> Self {
        Self {
            tile: 0,
            walls: Canvas::new(0, 0),
            canvas: Canvas::new(0, 0),
        }
    }

    fn draw(
        &mut self,
        frame: &mut Frame,
        board: &Board,
        pac: &Pacman,
        ghosts: &[Ghost],
        tick: u64,
        score: u32,
    ) {
        let area = frame.area();
        // biggest tile that fits. a cell is one pixel wide and two tall
        let t = (2..=8)
        .rev()
            .find(|&t| {
                board.width() * t <= area.width as i32
                && (board.height() * t + 1) / 2 + 2 <= area.height as i32
            })
                .unwrap_or(2);

        if t != self.tile {
            self.tile = t;
            self.walls = Canvas::new(board.width() * t, board.height() * t);
            gfx::draw_walls(
                &mut self.walls,
                board.width(),
                board.height(),
                t,
                |x, y| board.is_wall(x, y),
                gfx::WALL,
                gfx::WALL_FILL
            );
            self.canvas = Canvas::new(board.width() * t, board.height() * t);
        }
        let c = &mut self.canvas;
        c.copy_from(&self.walls);
        for (y, row) in board.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                match tile {
                    '.' => gfx::draw_dot(c, x as i32, y as i32, t),
                    'o' => gfx::draw_power(c, x as i32, y as i32, t),
                    _ => {}
                }
            }
        }

        let center = |tile: i32| (tile * t) as f32 + t as f32 / 2.0;
        let r = t as f32 * 0.8;
        let mouth = if pac.steps.is_multiple_of(2) { 0.05 } else { 0.22 };
        gfx::draw_pacman(c, centre(pac.x), centre(pac.y), r, pac.facing, mouth);
        for ghost in ghosts{
            gfx::draw_ghost(
                c,
                center(ghost.x),
                center(ghost.y),
                r,
                ghost.color,
                ghost.looking,
                tick,
            );
        }

        let rows = ((board.height() * t + 1) / 2) as u16;
        let board_area = Rect::new(area.x, area.y, (board.width() * t) as u16, rows).intersection(area);
        frame.render_widget(&self.canvas, board_area);
        let below = |line: u16| Rect::new(area.x, area.y + rows + line, area.width, 1).intersection(area);
        frame.render_widget(
            Paragraph::new(format!("score {}    pellets {}", score, board.pellets)),
            below(0),
        );
        frame.render_widget(Paragraph::new("arrows: move    q: quit"), below(1));
    }
}

fn spawn_pacman() -> Pacman {
    Pacman { x: 11, y: 14, dir: (0, 0)}
}

fn spawn_ghosts() -> Vec<Ghost> {
    vec![
        Ghost { x: 9, y: 9, color: Color::Red },
        Ghost { x: 11, y: 9, color: Color::Magenta },
        Ghost { x: 9, y: 10, color: Color::Cyan },
        Ghost { x: 11, y: 10, color: Color::LightRed },
    ]
}

fn centered(area: React, width: u16, height: u16) -> Rect {
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width.min(area.width),
        height.min(area.height),
    )
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut board = Board::new();
    let mut pac = Pacman {
        x: 11,
        y: 14,
        facing: Dir::Left,
        steps: 0,
    };
    board.eat(pac.x, pac.y);
    let mut ghosts = vec![
        Ghost::new(9, 9, gfx::GHOSTS[0]),
        Ghost::new(11, 9, gfx::GHOSTS[1]),
        Ghost::new(9, 10, gfx::GHOSTS[2]),
        Ghost::new(11, 10, gfx::GHOSTS[3]),
    ];

    let mut scene = Scene::new();
    let mut tick = 0;
    let mut score = 0;

    loop {

        tick += 1;
        terminal.draw(|frame| scene.draw(frame, &board, &pac, &ghosts, tick, score))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let dir = match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Up => Dir::Up,
                        KeyCode::Down => Dir::Down,
                        KeyCode::Left => Dir::Left,
                        KeyCode::Right = > Dir::Right,
_ => continue,
                    };
                    score += pac.step(&mut board, dir);
                    for ghost in &mut ghosts {
                        ghost.chase(&pac, &board);
                    }
                }
            }

        if over {
            continue;
        }

        if last_tick.elapsed() >= TICK {
            score += pac.step(&mut board);
            for ghost in &mut ghosts {
                ghost.chase(&pac, &board);
            }

            if ghosts.iter().any(|ghost| ghost.x == pac.x && ghost.y == pac.y) {
                lives -= 1;
                if lives == 0 {
                    over = true;
                } else {
                    pac = spawn_pacman();
                    ghosts = spawn_ghosts();
                }
            }
            last_tick = Instant::now();
        }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
