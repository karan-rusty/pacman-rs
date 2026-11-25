use crate::game::{Game, Pos, SUB};
use crate::gfx::{self, Canvas};
use crate::maze::{Maze, Tile, HEIGHT, WIDTH};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

const TILE_SIZES: std::ops::RangeInclusive<i32> = 3..=8;

pub struct Renderer {
    tile: i32,
    walls: Canvas,
    canvas: Canvas,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            tile: 0,
            walls: Canvas::new(0, 0),
            canvas: Canvas::new(0, 0),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, game: &Game) {
        let area = frame.area();
        let Some(t) = TILE_SIZES.rev().find(|&t| fits(t, area)) else {
            let t = *TILE_SIZES.start();
            let msg = format!(
                "make the terminal at least {}x{}, it is {}x{}",
                WIDTH * t,
                board_rows(t) + 2,
                area.width,
                area.height
            );
            let middle = Rect::new(area.x, area.y + area.height / 2, area.width, 1);
            frame.render_widget(Paragraph::new(msg).alignment(Alignment::Center), middle);
            return;
        };
        self.resize(t);
        self.paint(game);

        let width = (WIDTH * t) as u16;
        let rows = board_rows(t) as u16;
        let left = area.x + (area.width - width) / 2;
        let top = area.y + (area.height - rows - 2) / 2;
        frame.render_widget(&self.canvas, Rect::new(left, top + 1, width, rows));
        frame.render_widget(
            Paragraph::new(format!("SCORE {}", game.score)),
            Rect::new(left, top, width, 1),
        );
        frame.render_widget(
            Paragraph::new("arrows move   q quit")
                .style(Style::new().fg(Color::DarkGray))
                .alignment(Alignment::Center),
            Rect::new(left, top + 1 + rows, width, 1),
        );
    }

    fn resize(&mut self, t: i32) {
        if t == self.tile {
            return;
        }
        self.tile = t;
        let maze = Maze::new();
        self.walls = Canvas::new(WIDTH * t, HEIGHT * t);
        let is_wall = |x: i32, y: i32| maze.get(x, y) == Tile::Wall;
        gfx::draw_walls(
            &mut self.walls,
            WIDTH,
            HEIGHT,
            t,
            is_wall,
            gfx::Wall,
            gfx::WALL_FILL,
        );
        self.canvas = Canvas::new(WIDTH * t, HEIGHT * t);
    }

    fn paint(&mut self, game: &Game) {
        let t = self.tile;
        let c = &mut self.canvas;
        c.copy_from(&self.walls);

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                match game.maze.get(x, y) {
                    Tile::Dot => gfx::draw_dot(c, x, y, t),
                    Tile::Power => gfx::draw_power(c, x, y, t),
                    _ => {}
                }
            }
        }

        let r = t as f32 * 0.8;
        let pac = &game.pac;
        let (x, y) = to_pixels(pac.pos, t);
        gfx::draw_pacman(c, x, y, r, pac.dir, pac.mouth() * 0.22);,

        for (ghost, color) in game.ghosts.iter().zip(gfx::GHOSTS) {
            let (x, y) = to_pixels(ghost.pos, t);
            gfx::draw_ghost(c, x, y, r, color, ghost.dir, game.frame);
        }
    }
}

fn board_rows(t: i32) -> i32 {
    (HEIGHT * t + 1) / 2
}

fn fits(t: i32, area: React) -> bool {
    WIDTH * t <= area.width as i32 && board_rows(t) + 2 <= area.height as i32
}

fn to_pixels(pos: Pos, t: i32) -> (f32, f32) {
    let scale = t as f32 / SUB as f32;
    let half = t as f32 / 2.0;
    (pos.x as f32 * scale + half, pos.y as f32 * scale + half)
}
