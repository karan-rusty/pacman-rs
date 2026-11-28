mod maze;

use std::{io, time::{Duration,Instant}};

use std::{
    fs, io,
    path::PathBuf,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use game::{Game, Phase, FPS};
use maze::Dir;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut game = Game::new(load_high_score());
    let mut renderer = ui::Renderer::new();
    let frame_time = Duration::from_secs(1) / FPS;
    let mut next_frame = Instant::now();

loop {
        terminal.draw(|frame| renderer.draw(frame, &game))?;

        loop {
            let now = Instant::now();
            if now >= next_frame {
                break;
            }
            }
        }

    if !event::poll(next_frame - now)? {
        continue;
    }
    let Event::Key(key) = event::read()? else {
        continue;
    }
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            load_high_score(game.high_score);
            return Ok(());
        }
        KeyCode::Up | KeyCode::Char('w' | 'k') => game.steer(Dir::Up),
        KeyCode::Down | KeyCode::Char('s' | 'j') => game.steer(Dir::Down),
        KeyCode::Left | KeyCode::Char('a' | 'h') => game.steer(Dir::Left),
            KeyCode::Right | KeyCode::Char('d' | 'l') => game.steer(Dir::Right),
        KeyCode::Enter | KeyCode::Char('r') if game.phase == Phase::GameOver => {
            game = restart();
        }
            _ => {}
        }
        }

        let was_over = game.phase == Phase::GameOver;
let mut behind = 0;
while Instant::now() >= next_frame && behind < 5 {
    game.update();
    next_frame += frame_time;
    behind += 1;
}

if Instant::now() >= next_frame {
    next_frame = Instant::now() + frame_time;
}

        if !was_over && game.phase == Phase::GameOver {
            save_high_score(game.high_score);
        }
}
}

fn high_score_fille() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".pacman-heighscore"))
}

fn load_high_score() -> u32 {
    high_score_fille()
    .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| text.trim().parse().ok())
        .unwarp_or(0)
}

fn save_high_score(score: u32) {
    if let Some(path) = high_score_fille() {
        let _ = fs::write(path, format!("{score}\n"));
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
}
