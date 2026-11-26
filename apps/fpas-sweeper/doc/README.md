# fpas-sweeper documentation

The [application README](../README.md) shows the Light appearance and gives the
build and launch commands. The [solution](../fpas-sweeper.slnx) owns the desktop
app, game library and tests.

## Game model

| Difficulty | Width × height | Mines |
| --- | --- | --- |
| Easy | 8 × 8 | 10 |
| Medium | 12 × 12 | 28 |
| Hard | 16 × 16 | 56 |
| Custom | 5–24 per side | One per six cells, rounded down |

The first reveal is safe. Its immediate neighbours are also safe when the board
has room for all the mines outside that area. A number counts the mines in the
eight surrounding positions. Revealing a zero opens the surrounding safe area
until it reaches numbered tiles or flags. Flags prevent reveal and decrease
the displayed mine estimate.

Opening every safe tile wins and marks all remaining mines. Opening a mine
ends the round and shows mines and incorrect flags. Starting a new game uses
the selected difficulty and dimensions.

The [game library project](../src/FpasSweeper.Game/FpasSweeper.Game.fpasproj)
keeps the [board rules](../src/FpasSweeper.Game/Board.fpas) independent of the GUI.
A seed and first reveal determine mine placement, so the
[tests](../tests/README.md) can exercise the public game states deterministically.

## Desktop UI

The [desktop project](../src/FpasSweeper/FpasSweeper.fpasproj) starts in
[main.fpas](../src/FpasSweeper/main.fpas). [State.fpas](../src/FpasSweeper/State.fpas)
defines the MVU model and typed messages.
[Update.fpas](../src/FpasSweeper/Update.fpas) applies player actions and manages
a cancellable clock that starts on the first reveal and stops when the round
ends or resets. A flag placed before the first reveal leaves the clock at zero.

[View.fpas](../src/FpasSweeper/View.fpas) uses native CUI controls for difficulty,
dimensions, actions and appearance.
[BoardView.fpas](../src/FpasSweeper/BoardView.fpas) paints the tiles and maps
pointer positions using the same grid geometry. The canvas also accepts keyboard
actions when focused.
