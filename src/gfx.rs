use std::f32::consts::PI;

use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

use crate::maze::Dir;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl From<Rgb> for Color {
    fn from(c: Rgb) -> Self {
        Color::Rgb(c.0, c.1, c.2)
    }
}

pub const BLACK: Rgb = Rgb(0, 0, 0);
pub const WALL: Rgb = Rgb(33, 33, 255);
pub const WALL_FILL: Rgb = Rgb(8, 8, 56);
pub const DOT: Rgb = Rgb(255, 184, 151);
pub const PACMAN: Rgb = Rgb(255, 255, 0);
pub const EYE_WHITE: Rgb = Rgb(222, 222, 255);
pub const PUPIL: Rgb = Rgb(33, 33, 255);
pub const SCARED: Rgb = Rgb(33, 33, 255);
pub const SCARED_FACE: Rgb = Rgb(255, 184, 151);
pub const FLASH: Rgb = Rgb(222, 222, 255);
pub const FLASH_FACE: Rgb = Rgb(255, 0, 0);
pub const WALL_FLASH: Rgb = Rgb(222, 222, 255);
pub const WALL_FLASH_FILL: Rgb = Rgb(70, 70, 110);
pub const DOOR: Rgb = Rgb(255, 184, 222);
pub const CHERRY: Rgb = Rgb(255, 0, 0);
pub const STEM: Rgb = Rgb(222, 151, 81);
pub const GHOSTS: [Rgb; 4] = [
    Rgb(255, 0, 0),
    Rgb(255, 184, 255),
    Rgb(0, 255, 255),
    Rgb(255, 184, 82),
];

pub struct Canvas {
    pub width: i32,
    pub height: i32,
    pixels: Vec<Rgb>,
}

impl Canvas {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            pixels: vec![BLACK; (width * height) as usize],
        }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        let inside = x >= 0 && y >= 0 && x < self.width && y < self.height;
        inside.then(|| (y * self.width + x) as usize)
    }

    pub fn get(&self, x: i32, y: i32) -> Rgb {
        self.index(x, y).map_or(BLACK, |i| self.pixels[i])
    }

    pub fn set(&mut self, x: i32, y: i32, color: Rgb) {
        if let Some(i) = self.index(x, y) {
            self.pixels[i] = color;
        }
    }

    pub fn copy_from(&mut self, other: &Canvas) {
        self.pixels.copy_from_slice(&other.pixels);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgb) {
        for py in y..y + h {
            for px in x..x + w {
                self.set(px, py, color);
            }
        }
    }

    pub fn fill_shape(
        &mut self,
        cx: f32,
        cy: f32,
        reach: f32,
        color: Rgb,
        inside: impl Fn(f32, f32) -> bool,
    ) {
        let (x0, x1) = ((cx - reach).floor() as i32, (cx + reach).ceil() as i32);
        let (y0, y1) = ((cy - reach).floor() as i32, (cy + reach).ceil() as i32);
        for y in y0..=y1 {
            for x in x0..=x1 {
                if inside(x as f32 + 0.5 - cx, y as f32 + 0.5 - cy) {
                    self.set(x, y, color);
                }
            }
        }
    }

    pub fn fill_circle(&mut self, cx: f32, cy: f32, r: f32, color: Rgb) {
        self.fill_shape(cx, cy, r, color, |dx, dy| dx * dx + dy * dy <= r * r);
    }

    pub fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Rgb) {
        let n = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0) as i32;
        for i in 0..=n {
            let k = i as f32 / n as f32;
            let (x, y) = (x0 + (x1 - x0) * k, y0 + (y1 - y0) * k);
            self.set(x.floor() as i32, y.floor() as i32, color);
        }
    }

    pub fn fill_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, color: Rgb) {
        let mut hit = false;
        let (x0, x1) = ((cx - rx).floor() as i32, (cx + rx).ceil() as i32);
        let (y0, y1) = ((cy - ry).floor() as i32, (cy + ry).ceil() as i32);
        for y in y0..=y1 {
                for x in x0..=x1 {
                    let dx = (x as f32 + 0.5 - cx) / rx;
                    let dy = (y as f32 + 0.5 - cy) / ry;
                    if dx * dx + dy * dy <= 1.0 {
                        self.set(x, y, color);
                        hit = true;
                    }
                }
            }
            if !hit {
                self.set(cx.floor() as i32, cy.floor() as i32, color);
            }
        }
    }

    impl Widget for &Canvas {
        fn render(self, area: Rect, buf: &mut Buffer) {
            let rows = ((self.height + 1) / 2).min(area.height as i32);
            let cols = self.width.min(area.width as i32);
            for row in 0..rows {
                for col in 0..cols {
                    let top = self.get(col, row * 2);
                    let bottom = self.get(col, row * 2 + 1);
                    let Some(cell) = buf.cell_mut((area.x + col as u16, area.y + row as u16)) else {
                        continue;
                    };
                    if top == bottom {
                        cell.set_char(' ').set_fg(Color::Reset).set_bg(top.into());
                    } else {
                        cell.set_char('▀').set_fg(top.into()).set_bg(bottom.into());
                    }
                }
            }
        }
    }

fn unit(dir: Dir) -> (f32, f32) {
    let (x, y) = dir.delta();
    (x as f32, y as f32)
}

pub fn draw_walls(
    c: &mut Canvas,
    cols: i32,
    rows: i32,
    t: i32,
    is_wall:impl Fn(i32, i32) -> bool,
    edge: Rgb,
    fill: Rgb,
) {
    let (w, h) = (cols * t, rows * t);
    let r = (t / 3).max(1);
    let wall_px = |x: i32, y: i32| is_wall(x.clamp(0, w - 1) / t, y.clamp(0, h - 1) / t);

    let mut solid = vec![false; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            solid[(y * w + x) as usize] = (-r..=r)
            .all(|dy| (-r..=r).all(|dx| dx * dx + dy * dy > r * r || wall_px(x + dx, y + dy)));
        }
    }

    let at = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && solid[(y * w + x) as usize];

    let mut inner = vec![false; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            inner[(y * w + x) as usize] = (-1..=1).all(|dy| (-1..=1).all(|dx| at(x + dx, y + dy)));
        }
    }
    let mut filled = vec![false; (w * h) as usize];
    let mut seen = vec![false; (w * h) as usize];
    for start in 0..(w * h) as usize {
        if !inner[start] || seen[start] {
            continue;
        }
        seen[start] = true;
        let mut blob = vec![start];
        let mut next = 0;
        while next < blob.len() {
            let (x, y) = (blob[next] as i32 % w, blob[next] as i32 / w);
            next += 1;
            for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let i = (ny * w + nx) as usize;
                if inner[i] && !seen[i] {
                    seen[i] = true;
                    blob.push(i);
                }
            }
        }
        if blob.len() as i32 >= 2 * t {
            for i in blob {
                filled[i] = true;
            }
        }
    }
    for y in 0..h {
        for x in 0..w {
            if !at(x, y) {
                continue;
            }
            let (up, down, left, right) = (at(x, y - 1), at(x, y + 1), at(x - 1, y), at(x + 1, y));
            if (!up || !down) && (!left || !right) {
                continue;
            }
            let filled = filled[(y * w + x) as usize];
            c.set(x, y, if filled { fill } else { edge });
        }
    }
}

pub fn draw_door(c: &mut Canvas, tx: i32, ty: i32, t: i32) {
    let h = (t / 4).max(1);
    c.fill_rect(tx * t, ty * t + (t - h) / 2, t, h, DOOR);
}

pub fn draw_dot(c: &mut Canvas, tx: i32, ty: i32, t: i32) {
    let s = if t % 2 == 0 { 2 } else { 1 };
    let o = (t - s) / 2;
    c.fill_rect(tx * t + o, ty * t + o, s, s, DOT);
}

pub fn draw_power(c: &mut Canvas, tx: i32, ty: i32, t: i32) {
    let half = t as f32 / 2.0;
    let (x, y) = ((tx * t) as f32 + half, (ty * t) as f32 + half);
    c.fill_circle(x, y, t as f32 * 0.45, DOT);
}

pub fn draw_pacman(c: &mut Canvas, cx: f32, cy: f32, r: f32, facing: Dir, mouth: f32) {
    let (fx, fy) = unit(facing);
    let half = mouth * PI;
    c.fill_shape(cx, cy, r, PACMAN, |dx, dy| {
        if dx * dx + dy * dy > r * r {
            return false;
        }
        let ahead = dx * fx + dy * fy;
        let aside = (dx * fy - dy * fx).abs();
        aside.atan2(ahead) >= half
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GhostLook {
    Body(Rgb),
    Scared {
        flash: bool,
    },
    Eyes,
}

pub fn draw_ghost(
    c: &mut Canvas,
    cx: f32,
    cy: f32,
    r: f32,
    look: GhostLook,
    facing: Dir,
    frame: u64,
){
    let shift = if (frame / 8).is_multiple_of(2) {
        0.0
    } else {
            0.5
        };
    let body = |dx, dy| ghost_body(dx, dy, r, shift);
    match look {
    GhostLook::Body(color) => {
    c.fill_shape(cx, cy, r, color, body);
    draw_eyes(c, cx, cy, r, facing);
    }
    GhostLook::Scared { flash } => {
        let (color, face) = if flash {
(FLASH, FLASH_FACE)
        } else {
(SCARED, SCARED_FACE)
        };
        c.fill_shape(cx, cy, r, color, body);
        draw_scared_face(c, cx, cy, r, face);
    }
    GhostLook::Eyes => draw_eyes(c, cx, cy, r, facing),
}
}

fn ghost_body(dx: f32, dy: f32, r: f32, shift: f32) -> bool {
    if dy < 0.0 {
        return dx * dx + dy * dy <= r * r;
    }
    if dx.abs() > r || dy > r {
        return false;
    }
    if dy < r * 0.6 {
        return true;
    }
    let feet = ((dx + r) / (2.0 * r) * 3.0 + shift).fract();
    (feet - 0.5).abs() < 0.3
}

fn draw_eyes(c: &mut Canvas, cx: f32, cy: f32, r: f32, facing: Dir) {
    let (lx, ly) = unit(facing);
    for side in [-1.0, 1.0] {
        let ex = cx + side * r * 0.4 + lx * r * 0.1;
        let ey = cy - r * 0.25 + ly * r * 0.1;
        if r < 4.5 {
            let (x, y) = (ex.floor() as i32, ey.floor() as i32);
            let (dx, dy) = facing.delta();
            c.set(x, y, EYE_WHITE);
            c.set(x + dx, y + dy, PUPIL);
        } else {
            c.fill_ellipse(ex, ey, r * 0.28, r * 0.36, EYE_WHITE);
            c.fill_ellipse(
                ex + lx * r * 0.14,
                ey + ly * r * 0.18,
                r * 0.16,
                r * 0.16,
                PUPIL,
            );
        }
    }
}

/// two dot eyes and a wobbly mouth
fn draw_scared_face(c: &mut Canvas, cx: f32, cy: f32, r: f32, face: Rgb) {
    for side in [-1.0, 1.0] {
        c.fill_ellipse(
            cx + side * r * 0.38,
            cy - r * 0.25,
            r * 0.14,
            r * 0.14,
            face,
        );
    }
    let mouth_y = (cy + r * 0.3).floor() as i32;
    let (x0, x1) = (
    (cx - r * 0.65).round() as i32,
    (cx + r * 0.65).round() as i32,
);
    for x in x0..x1 {
        c.set(x, mouth_y - (x & 1), face);
    }
}

pub fn draw_cherry(c: &mut Canvas, cx: f32, cy: f32, t: f32) {
    let top = (cx + t * 0.3, cy - t * 0.6);
    c.draw_line(cx - t * 0.3, cy + t * 0.1, top.0, top.1, STEM);
    c.draw_line(cx + t * 0.3, cy + t * 0.2, top.0, top.1, STEM);
    c.fill_circle(cx - t * 0.3, cy + t * 0.3, t * 0.3, CHERRY);
    c.fill_circle(cx + t * 0.3, cy + t * 0.4, t * 0.3, CHERRY);
}
