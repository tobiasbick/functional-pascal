# fpastris

A falling-block puzzle game written in Functional Pascal with `Std.Tui`.
The dark blue-gray palette, cyan accents, rounded frames and keyboard hints
follow the other [FPAS apps](../README.md) and the terminal interfaces collected
at [Terminal Apps](https://terminal-apps.dev/).

```sh
fpas run apps/fpastris/fpastris.fpasprj
```

Use a Unicode terminal with at least **62 columns × 26 rows**. Truecolor is
recommended. The game is centered in larger windows. Shrinking the terminal
below the minimum pauses play; resize and press Enter to resume.

## Controls

| Key | Action |
| --- | --- |
| Enter | Start, pause, resume or play again after game over |
| Left / Right or A / D | Move |
| Up or X | Rotate clockwise |
| Z | Rotate counterclockwise |
| Down or S | Soft drop |
| Space | Hard drop and lock |
| C | Hold or swap, once per piece |
| P or Escape | Pause / resume |
| R | Start a new game |
| Q, Ctrl+C or Alt+X | Quit |

## Rules

- A 10 × 20 visible board, seven tetrominoes, shuffled seven-piece bags,
  three upcoming pieces and a ghost showing the landing position.
- Rotation tries the original position, horizontal offsets of one and two
  cells, then upward offsets of one and two cells. These are simple wall/floor
  kicks, not the official SRS rules.
- Grounded pieces lock after 500 ms. Successful grounded moves and rotations
  reset the delay at most 15 times per piece. Hard drop locks immediately.
- Clearing one, two, three or four rows awards 100, 300, 500 or 800 points
  multiplied by the current level. Soft drop adds one point per cell and hard
  drop adds two. Every ten cleared rows increases the level and fall speed.
- A blocked spawn or a piece locking in the two hidden spawn rows ends the game.
- The best score survives restarts within the current session. It is not saved
  to disk. There is no network service or configuration file.

## Structure and checks

`Pieces`, `Model` and `Engine` implement the game rules. `Update` handles input
and one-shot TUI ticks; `View`, `Drawing` and `Theme` build the colored cell grid.
The terminal lifecycle and damage rendering belong to `Std.Tui`.

```sh
fpas check apps/fpastris/fpastris.fpasprj
fpas test tests/suite.fpasprj --filter fpastris
```

The tests cover game rules and headless TUI input, rendered cells, timer
requests, pause, resize, restart and quit.
