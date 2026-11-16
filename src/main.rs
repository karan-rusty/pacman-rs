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
    style::{Color, Style},
    widgets::Widget,
    Terminal,
};

struct Maze;

impl Widget for Maze {
    fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        for (y, row) in maze::MAZE.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let (symbol, color) = match ch {
                    '#' => ("█", Color::Blue),
                    '.' => (".", Color::Gray),
                    'o' => ("●", Color::Yellow),
                    _ => (" ", Color::Reset),
                };
                buf.set_string(
                    area.x + 2 + x as u16,
                    area.y + 1 + y as u16,
                    symbol,
                    Style::default().fg(color),
                );
            }
        }
    }
}

fm main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    loop {
        terminal.draw(|frame| {
            frame.render_widget(Maze, frame.area());
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
