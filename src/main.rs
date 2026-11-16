mod maze;

use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::crosstermBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Paragraph, Widget},
    Terminal,
};

const OFFSET_X: u16 = 2;
const OFFSET_Y: u16 = 1;

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
        Self {tiles, pellets}
    }

    fn width(&self) -> u16 {
        self.tiles[0].len() as u16
    }

    fn height(&self) -> u16 {
        self.tiles[0].len() as u16
    }

    fn height(&self) -> u16 {
        self.tiles.len() as u16
    }

    fn tile(&self, x: i32, y: i32) -> char {
        if x < 0 || y < 0 || x >= self.width() as i32 || y >= self.height() as i32 {
            return '#';
        }
        self.tiles[y as usize][x as usize]
    }

    fn is_wall(&self, x: i32, y: i32) -> bool {
        self.tile(x, y) == '#'
    }

    fn eat(&mut self, x: i32, y: i32) -> u32 {
        let points = match self.tile(x,y) {
            '.' => 10,
            'o' => 50,
            _ => return 0,
        };
        self.tiles[y as usize][x as usize] = ' ';
        self.pellets -= 1;
        points
    }
}

impl Widget for &Board {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let (symbol, color) = match tile {
                    '#' => ("█", Color::Blue),
                    '.' => ("·", Color::Gray),
                    'o' => ("●", Color::Yellow),
                    _ => (" ", Color::Reset),
                };
                buf.set_string(
                    area.x + OFFSET_X + x as u16,
                    area.y + OFFSET_Y + y as u16,
                    symbol,
                    Style::default().fg(color),
                );
            }
        }
    }
}

struct Pacman {
    x: i32,
    y: i32,
}

impl Pacman {
    fn step(&mut self, board: &mut Board, dx: i32, dy: i32) -> u32 {
        if board.is_wall(self.x + dx, self.y + dy) {
            return 0;
        }
        self.x += dx;
        self.y += dy;
        board.eat(self.x, self.y)
    }
}

impl Widget for &Pacman {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.x + OFFSET_X + self.x as u16,
            area.y + OFFSET_Y + self.y as u16,
            "@",
            Style::default().fg(Color::Yellow),
        );
    }
}

fm main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut board = Board::new();
    let mut pac = Pacman { x: 11, y: 14 };
    let mut score = 0;
    board.eat(pac.x, pac.y);

    loop {
        terminal.draw(|frame| {
            frame.render_widget(&board, frame.area());
            frame.render_widget(&pac, frame.area());
            frame.render_widget(
                Paragraph::new(format!("score {}    pellets {}", score,board.pellets)),
                Rect::new(OFFSET_X, board.height() + 2, 40, 1),
            );
            frame.render_widget(
                Paragraph::new("arrows: move     q: quit"),
                Rect::new(OFFSET_X, board.height() + 3, 40, 1),
            );
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Up => score += pac.step(&mut board, 0, -1),
                        KeyCode::Down => score += pac.step(&mut board, 0, 1),
                        KeyCode::Left => score += pac.step(&mut board, 1, 0),
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
