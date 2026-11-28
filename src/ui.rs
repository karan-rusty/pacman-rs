use crate::game::{Game, GhostState, Phase, Pos, CLEAR_FRAMES, DEATH_FRAMES, SUB};
use crate::gfx::{self, Canvas, GhostLook};
use crate::maze::{Dir, Maze, Tile, HEIGHT, MESSAGE_ROW, WIDTH};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

const TILE_SIZES: std::ops::RangeInclusive<i32> = 3..=8;
const FOOTER: i32 = 2;
const DEATH_FREEZE: u32 = 45;
const CLEAR_HOLD: u32 = 30;

const YELLOW: Color = Color::Rgb(255, 255, 0);
const RED: Color = Color::Rgb(255, 0, 0);
const CYAN: Color = Color::Rgb(0, 255, 255);
const WHITE: Color = Color::Rgb(222, 222, 255);

pub struct Renderer {
    tile: i32,
    walls: Canvas,
    walls_flash: Canvas,
    canvas: Canvas,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            tile: 0,
            walls: Canvas::new(0, 0),
            walls_flash: Canvas::new(0, 0),
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
        let board = Rect::new(left, top + 1, width, rows);
        frame.render_widget(&self.canvas, board);
        hud(frame, game, Rect::new(left, top, width, 1));
        self.messages(frame, game, board);
        help(frame, game, Rect::new(left, top + 1 + rows, width, 1));
    }

    fn resize(&mut self, t: i32) {
        if t == self.tile {
            return;
        }
        self.tile = t;
        let maze = Maze::new();
    let (w, h) = (WIDTH * t, (HEIGHT + FOOTER) * t);
        let is_wall = |x: i32, y: i32| maze.get(x, y) == Tile::Wall;
    let mut walls = Canvas::new(w, h);
    let mut flash = Canvas::new(w, h);
        gfx::draw_walls(
            &mut walls,
            WIDTH,
            HEIGHT,
            t,
            is_wall,
            gfx::WALL,
            gfx::WALL_FILL,
        );
    gfx::draw_walls(
        &mut flash,
        WIDTH,
        HEIGHT,
        t,
        is_wall,
        gfx::WALL_FLASH,
        gfx::WALL_FLASH_FILL,
    );
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if maze.get(x, y) == Tile::Door {
                gfx::draw_door(&mut walls, x, y, t);
                }
            }
        }
    self.walls = walls;
    self.walls_flash = flash;
    self.canvas = Canvas::new(w, h);
    }

    fn paint(&mut self, game: &Game) {
        let t = self.tile;
    let flash = matches!(game.phase, Phase::Cleared(n)
if n < CLEAR_FRAMES - CLEAR_HOLD && (n / 12).is_multiple_of(2));
    self.canvas.copy_from(if flash {
        &self.walls_flash
    } else {
            &self.walls
        }
    );
        let c = &mut self.canvas;

    let blink_off = game.phase == Phase::Playing && (game.frame / 10) % 2 == 1;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                match game.maze.get(x, y) {
                    Tile::Dot => gfx::draw_dot(c, x, y, t),
                Tile::Power if !blink_off => gfx::draw_power(c, x, y, t),
                    _ => {}
                }
            }
        }

    let tf = t as f32;
    let footer_y = (HEIGHT as f32 + 1.0) * tf;
    for i in 0..game.lives.min(6) {
        let x = (1.5 + i as f32 * 1.8) * tf;
        gfx::draw_pacman(c, x, footer_y, tf * 0.7, Dir::Left, 0.2);
    }
    for i in 0..game.level.min(7) {
        let x = (WIDTH as f32 - 1.5 - i as f32 * 1.8) * tf;
        gfx::draw_cherry(c, x, footer_y, tf);
    }

    let r = tf * 0.8;
        let pac = &game.pac;
    let mouth = match game.phase {
        Phase::Playing => Some(pac.mouth() * 0.22),
    Phase::Ready(_) | Phase::Cleared(_) => Some(0.0),
    Phase::Dying(n) => {
        let since = DEATH_FRAMES - n;
        if since < DEATH_FREEZE {
            Some(0.0)
        } else {
            let fold = (since - DEATH_FREEZE) as f32 / (DEATH_FRAMES - DEATH_FREEZE - 20) as f32;
            (fold < 1.0).then_some(0.05 + 0.95 * fold)
        }
    }
    Phase::Eating { .. } | Phase::GameOver => None,
    };
    if let Some(mouth) = mouth {
        let facing = if matches!(game.phase, Phase::Dying(_)) {
            Dir::Up
        } else {
            pac.dir
        };
        let (x, y) = to_pixels(pac.pos, t);
        for x in wrapped(x, t) {
            gfx::draw_pacman(c, x, y, r, facing, mouth);
        }
    }

    let ghosts_shown = match game.phase {
        Phase::Ready(_) | Phase::Playing | Phase::Eating { .. } => true,
        Phase::Dying(n) => DEATH_FRAMES - n < DEATH_FREEZE,
        Phase::Cleared(_) | Phase::GameOver => false,
    };
    if !ghosts_shown {
        return;
    }
    for (i, ghost) in game.ghosts.iter().enumerate() {
        // the one just eaten is replaced by its points for a moment
        if matches!(game.phase, Phase::Eating { ghost, .. } if ghost == i) {
            continue;
        }
        let look = match ghost.state {
            GhostState::Eyes | GhostState::Entering => GhostLook::Eyes,
            _ if ghost.scared => GhostLook::Scared {
                flash: game.scared_flashing(),
            },
            _ => GhostLook::Body(gfx::GHOSTS[i]),
    };
            let (x, y) = to_pixels(ghost.pos, t);
            for x in wrapped(x, t) {
                gfx::draw_ghost(c, x, y, r, look, ghost.dir, game.frame);
            }
        }
    }

fn messages(&self, frame: &mut Frame, game: &Game, board: Rect) {
    let t = self.tile;
    let (text, color) = match game.phase {
        _ if game.paused => ("PAUSED", WHITE),
        Phase::Ready(_) => ("READY!", YELLOW),
        Phase::GameOver => ("GAME OVER", RED),
        Phase::Eating { ghost, points, .. } => {
            let (px, py) = to_pixels(game.ghosts[ghost].pos, t);
            let cell_y = board.y + (py / 2.0) as u16;
            let text = points.to_string();
            let cell_x = (board.x + px as u16).saturating_sub(text.len() as u16 / 2);
            put(frame, &text, CYAN, cell_x, cell_y, board);
            return;
        }
        _ => return,
    };
    let cell_y = board.y + ((MESSAGE_ROW * t + t / 2) / 2) as u16;
    let cell_x = board.x + (board.width - text.len() as u16) / 2;
    put(frame, text, color, cell_x, cell_y, board);
}
}

fn put(frame: &mut Frame, text: &str, color: Color, x: u16, y: u16, inside: Rect) {
    let style = Style::new()
        .fg(color)
        .bg(Color::Black)
        .add_modifier(Modifier::BOLD);
    let area = Rect::new(x, y, text.len() as u16, 1).intersection(inside);
    frame.render_widget(Paragraph::new(text).style(style), area);
}

fn hud(frame: &mut Frame, game: &Game, area: Rect) {
    let label = Style::new().fg(WHITE).add_modifier(Modifier::BOLD);
    let value = Style::new().fg(YELLOW);
let pair = |name: &'static str, n: u32| {
        Line::from(vec![
            Span::styled(name, label),
            Span::styled(n.to_string(), value),
        ])
    };
    frame.render_widget(Paragraph::new(pair("SCORE ", game.score)), area);
    frame.render_widget(
        Paragraph::new(pair("HIGH SCORE ", game.high_score)).alignment(Alignment::Center),
        area,
    );
    frame.render_widget(
Paragraph::new(pair("LEVEL ", game.level)).alignment(Alignment::Right),
        area,
    );
}

fn help(frame: &mut Frame, game: &Game, area: Rect) {
    let text = match game.phase {
        Phase::GameOver => "enter play again   q quit",
  _ => "arrows or wasd move   p pause   q quit",
    };
    let style = Style::new().fg(Color::DarkGray);
    frame.render_widget(
        Paragraph::new(text)
        .style(style)
            .alignment(Alignment::Center),
        area,
    );
}

fn board_rows(t: i32) -> i32 {
    ((HEIGHT + FOOTER) * t + 1) / 2
}

fn fits(t: i32, area: Rect) -> bool {
    WIDTH * t <= area.width as i32 && board_rows(t) + 2 <= area.height as i32
}

fn to_pixels(pos: Pos, t: i32) -> (f32, f32) {
    let scale = t as f32 / SUB as f32;
    let half = t as f32 / 2.0;
    (pos.x as f32 * scale + half, pos.y as f32 * scale + half)
}

fn wrapped(x: f32, t: i32) -> impl Iterator<Item = f32> {
    let span = (WIDTH * t) as f32;
    let margin = t as f32;
    [x, x - span, x + span]
        .into_iter()
        .filter(move |&x| x > -margin && x < span + margin)
}
