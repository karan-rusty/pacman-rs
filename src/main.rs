mod maze;

use std::{io, time::{Duration,Instant}};

use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use game::{Game, FPS};
use maze::Dir;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut game = Game::new();
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
            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
            KeyCode::Up => game.steer(Dir::Up),
            KeyCode::Down => game.steer(Dir::Down),
            KeyCode::Right => game.steer(Dir::Right),
            _ => {}
        }
        }

let mut behind = 0;
while Instant::now() >= next_frame && behind < 5 {
    game.update();
    next_frame += frame_time;
    behind += 1;
}

if Instant::now() >= next_frame {
    next_frame = Instant::now() + frame_time;
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
