# CLI

The crate `minigame-cli`, built as the binary `minigame`: the games for a person, and one round on a display of its own for an agent.

## Modules

| Module | What is in it |
|---|---|
| `main` | The command line |
| `window` | A desktop window, a keyboard, and a mouse, handed to the engine's `Shell` |
| `screen` | One round on a virtual X display that an agent works |

## Commands

| Command | What it does |
|---|---|
| `play [GAME]` | Opens the window on the menu, or straight on one game |
| `list` | Every game, its title, its limit, and what it is scored on |
| `prompt GAME` | What a player is told about one game |
| `task GAME` | What an agent on a screen is told: `session::task` at `--clock` or `--ticks` |
| `shot GAME` | One frame written to a PNG, the way an agent would receive it |
| `screen GAME` | One round on a virtual display, run in the game image |

`minigame` with no command is `play` with no game.

| Flag | Taken by | What it does | Default |
|---|---|---|---|
| `--seed N` | `play`, `shot`, `screen` | The layout | `play`: one drawn from the time; otherwise 0 |
| `--clock RATE` | `play`, `task`, `screen` | Ticks of game time per second of the player's. `task` and `screen` take 1 to 60. | 60 |
| `--ticks [N]` | `task`, `screen` | Instead of `--clock`: the game stands still, and each press of F1 lets `N` ticks pass, from 1 to 60, or 1 where `N` is left out | off, so the pace is `--clock` |
| `--countdown` | `play`, `shot` | Draws the limit counting down | off |
| `--after N` | `shot` | Ticks of game time let pass before the frame is taken | 0 |
| `--brief` | `shot` | The instructions screen instead of the game | off |
| `--out FILE` | `shot` | Where the PNG goes | `shot.png` |

## The Window

- `minifb` opens a window at twice the picture's size and reads the keyboard and mouse once a frame.
- Keys go to the shell as `down` and `up`, repeats included; the mouse as `move`, `press`, and `release` in picture pixels.
- Each frame the shell is told the wall time since the last, draws the picture, and gives the window its title.
- A round's end prints the game, the outcome, the game seconds and, where the game has one, the seed.

## The Screen

`screen` is what the game image runs, and the only command that needs `Xvfb`.

- It starts `Xvfb` on `:0` at 640×480 and 24 bits, with no key repeat, TCP, or shared memory, and waits until it is ready.
- A borderless, untitled window covers the display, so a picture of the display is the frame.
- `ready` goes to standard output once the first frame is up, and the round starts on `go` from standard input. A further line on standard input, or its closing, ends the round as it stands.
- A thread reads each event from the display as it arrives.
- At `--clock`, a tick is due every `1 / clock` seconds after `go` on a monotonic clock. Each event lands on the first tick not yet due when it was read, after any ticks that came due before it; anything before `go` lands on tick 0. Late ticks are played late, never dropped, and of a run of them only the last is drawn. F1 does nothing.
- At `--ticks N`, the game stands still until F1 goes down. Then every key and button held differently from the last press of F1 lands, the pointer's move first, `N` ticks pass, the frame is drawn, and `advanced` goes to standard output. A press and release between two presses of F1 lands nothing but where the pointer went.
- A key is read by the first keysym of its keycode, under the mapping as it stood at the last mapping change. Buttons 1 and 3 are left and right, movement is `move`, and anything no game reads is dropped.
- At the end, standard output has every action that landed, one to a line as its tick and a line of the action language, then `over`, the outcome, and the last tick. The lines read back with `Action::parse` and are the record `replay::replay` plays.
- An error from the display ends the round with an error rather than a result.
