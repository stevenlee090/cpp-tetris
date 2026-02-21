# Tetris in Rust

A fully working terminal Tetris game implemented in Rust using [ratatui](https://github.com/ratatui-org/ratatui) for TUI rendering.

> Originally a C++ / ncurses prototype — re-implemented from scratch in Rust by **Claude Sonnet 4.6** using **Claude Code v2.1.50**.

---

## Features

- All 7 standard tetrominoes (I, S, Z, O, T, L, J) with per-piece colors
- Rotation, collision detection, and line clearing
- Hard drop (Space) and soft drop (↓)
- Score tracking: +25 per piece placed, bonus for multi-line clears
- Speed increases every 10 pieces (up to a cap)
- Next-piece preview
- Pause / resume
- Game-over screen with final score
- Clean terminal restore on exit

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2021)

## Build & Run

```bash
cargo build
cargo run
```

## Controls

| Key       | Action       |
|-----------|--------------|
| `←` `→`  | Move         |
| `↑`       | Rotate       |
| `↓`       | Soft drop    |
| `Space`   | Hard drop    |
| `p`       | Pause/Resume |
| `q` / `Esc` | Quit       |

## Scoring

| Event              | Points              |
|--------------------|---------------------|
| Piece locked       | +25                 |
| 1 line cleared     | +200                |
| 2 lines cleared    | +400                |
| 3 lines cleared    | +800                |
| 4 lines cleared    | +1600               |

## Project Structure

```
├── Cargo.toml
└── src/
    ├── main.rs   — terminal init/cleanup, game loop, input handling
    ├── game.rs   — game state, tetrominoes, physics, scoring
    └── ui.rs     — ratatui rendering (board, sidebar, overlays)
```

## Dependencies

| Crate       | Version | Purpose                     |
|-------------|---------|-----------------------------|
| ratatui     | 0.29    | TUI rendering               |
| crossterm   | 0.28    | Cross-platform terminal I/O |
| rand        | 0.8     | Random piece selection      |

---

*Generated with [Claude Sonnet 4.6](https://www.anthropic.com/claude) using [Claude Code](https://github.com/anthropics/claude-code) v2.1.50*
