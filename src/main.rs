mod maze;

use std::{io, time::{Duration,Instant}};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Clear, Paragraph, Widget},
    Terminal,
};

const OFFSET_X: u16 = 2;
const OFFSET_Y: u16 = 1;
const TICK: Duration = Duration::from_millis(130);

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

    fn width(&self) -> u16 {
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
    dir: (i32, i32),
}

impl Pacman {
    fn turn(&mut self, dx: i32, dy: i32) {
        self.dir = (dx, dy);
    }
    
    fn step(&mut self, board: &mut Board) -> u32 {
        if self.dir == (0,0) || board.is_wall(self.x + self.dir.0, self.y + self.dir.1) {
            return 0;
        }
        self.x += self.dir.0;
        self.y += self.dir.1;
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

struct Ghost {
    x: i32,
    y: i32,
    color: Color,
}

impl Ghost {
    fn chase(&mut self, pac: &Pacman, board: &Board) {
        let dx =  pac.x - self.x;
        let dy = pac.y - self.y;
        let moves = if dx.abs() >= dy.abs() {
            [
                (dx.signum(), 0),
                (0, dy.signum()),
                (0, -dy.signum()),
                (-dx.signum(), 0),
            ]
        } else {
            [
                (0, dy.signum()),
                (dx.signum(), 0),
                (-dx.signum(), 0),
                (0, -dy.signum()),
            ]
        };

        for (mx, my) in moves {
            if (mx, my) !== (0, 0) && !board.is_wall(self.x + mx, self.y + my) {
                self.x += mx;
                self.y += my;
                return;
            }
        }
    }
}

impl Widget for &Ghost {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.x + OFFSET_X + self.x as u16,
            area.y + OFFSET_Y + self.y as u16,
            "M",
            Style::default().fg(self.color),
        );
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
    let mut pac = spawn_pacman();
    let mut score = 0;
    let mut lives = 3;
    let mut over = false;
    board.eat(pac.x, pac.y);
    let mut ghosts = spawn_ghosts();
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| {
            frame.render_widget(&board, frame.area());
            if !over {
                frame.render_widget(&pac, frame.area());
            }
            for ghost in &ghosts {
                frame.render_widget(ghost, frame.area());
            }
            frame.render_widget(
                Paragraph::new(format!("score {}   pellets {}   lives {}",
                    score, board.pellets, lives
                ))
                    Rect::new(OFFSET_X, board.height() + 2, 60, 1),
            );
            frame.render_widget(
                Paragraph::new("arrows: turn     q: quit"),
                Rect::new(OFFSET_X, board.height() + 3, 40, 1),
            );

            if over {
                let popup = centered(frame.area(), 34, 5);
                frame.render_widget(Clear, popup);
                frame.render_widget(
                    Paragraph::new(format!("GAME OVER    score {}\n\nr: restart     q: quit", score))
                    .block(Block::bordered())
                        .alignment(Alignment::Center),
                    popup,
                );
            }
        })?;

        let timeout = TICK.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('r') if over => {
                            board = Board::new();
                            pac = spawn_pacman();
                            ghosts = spawn_ghosts();
                            score = 0;
                            lives = 3;
                            over = false;
                            board.eat(pac.x, pac.y);
                        }
                        KeyCode::Up => pac.turn(0, -1),
                        KeyCode::Down => pac.turn(0, 1),
                        KeyCode::Left => pac.turn(-1, 0),
                        KeyCode::Right => pac.turn(1, 0),
                        _ => {}
                        }
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
