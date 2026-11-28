# pacman-rs

Pac-Man in the terminal

## Features

- **Pixel graphics** — drawn with half-block characters, scales up to fit the terminal
- **Four ghosts** — red chases you, pink aims ahead of you, cyan works off red's position, orange gives up when it gets close
- **Scatter and chase** — ghosts switch between going for you and heading to their corners, and turn around on every switch
- **Power pellets** — ghosts turn blue and run, eat them in a row for 200, 400, 800, 1600
- **Ghost house and tunnel** — eaten ghosts run home as eyes, the side tunnel wraps around and slows ghosts down
- **Levels** — everything gets faster and scared time gets shorter
- **Lives** — 3 to start, one extra at 10,000
- **High score** — kept between runs in `~/.pacman-heighscore`

## Installation

```sh
cargo install --git https://github.com/karan-rusty/pacman-rs
```

## Usage

Run `pacman` to start. The terminal needs to be at least 75x35.

## Keybinds

| Key                       | Action                  |
| ------------------------- | ----------------------- |
| arrows / `wasd` / `hjkl`  | Move                    |
| `p`                       | Pause                   |
| `Enter` / `r`             | Play again on game over |
| `q` / `Esc`               | Quit                    |
