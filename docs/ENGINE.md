# Engine

The crate `minigame`: the picture, the action language, the clock, and the episode.

## Modules

| Module | What is in it |
|---|---|
| `frame` | `Frame`, the 640×480 buffer, and everything that can be drawn into it |
| `palette` | The colors every game draws with |
| `rng` | `Rng`, the only source of chance, and `take`, which deals several different things at once |
| `input` | `Key`, `Button`, `Action`, `Input` |
| `game` | `TICK_HZ`, `TICK`, `LIMIT`, `Spec`, `Scoring`, `Tag`, `Outcome`, and the `Game` trait |
| `session` | `Session`, one episode of one game, `Report`, and `Pace` and `task`, what an agent on a screen is told |
| `replay` | `replay`, which plays an episode again from its record |
| `shell` | `Shell`: the menu, the brief, the round, and the result |
| `catalog` | `Entry` and `CATALOG`, the table of every game |
| `games` | One module per game |
| `walk`, `board`, `maze`, `map`, `shape`, `aim`, `sky` | [What games hold in common](#what-games-hold-in-common) |
| `words` | `WORDS`, the bank of five-letter words |

## The Picture

`Frame` holds `WIDTH * HEIGHT` packed `0x00RRGGBB` words. Shapes are clipped at the buffer's edge; there is no anti-aliasing and no alpha.

| Drawing | What it is |
|---|---|
| `fill`, `rect`, `outline`, `disc`, `ring`, `line`, `poly` | The whole frame in one color, filled and outlined rectangles, discs, rings, thick lines, and polygons by the even-odd rule |
| `text`, `text_center` | An embedded 8×8 font at integer scales |
| `corners` | Corners spaced evenly around a circle, each with its own reach: triangles, pentagons, stars, diamonds |
| `wedge` | A triangle whose corners sit on its bounding box |
| `spun` | A shape given as points about the origin, placed and turned |
| `cross`, `crossed` | Two strokes meeting at a point, square on and turned |
| `bordered` | A filled box inside a border |
| `pips` | A centered row of dots with the first so many lit |
| `to_png` | The buffer encoded |

## The Action Language

An action is one line of text:

```
click|press|release [left|right] <x> <y>      move <x> <y>
down|up|tap <KEY>                             wait
```

- `Action::parse` refuses anything it does not recognize, including trailing words and `nan` or `inf`.
- `click`, `press`, and `release` move the pointer to their place first; `move` on its own drags whatever is held.
- `left` may be written before the numbers and `right` must be.
- `Key` covers the letters, the digits, space, enter, backspace, escape, the four arrows, and shift.

## Input

`Input` is what a game is handed each tick: the pointer, what is held, and what changed. A click or a tap is a press and a release in the same tick.

| Helper | What it answers |
|---|---|
| `axis` | Which of a pair of keys is held, as -1, 0, or 1 |
| `arrows` | `axis` for both axes |
| `heading` | `arrows` as a step of length one |
| `arrow` | The first of `ARROWS` that was pressed, for games that take one arrow a press |
| `lift_held`, `lift_pressed` | UP or SPACE, for a thrust that is held and a jump that is not |
| `typed` | The first key pressed this tick that stands for a character |

The pointer is drawn for games that use it: an open arrow whose tip is the point, colored differently while the left button is held, and absent until an action has placed it.

## What Games Hold In Common

| Helper | What it does |
|---|---|
| `walk::walked` | Moves a body a full stride the way asked, or along whichever half of that way is not walled |
| `walk::slid` | One axis of a move, ending the whole step along or against what stopped it |
| `walk::toward`, `walk::eased` | Chase a goal in both axes or one, landing exactly on it |
| `walk::bounced`, `walk::bumped` | A round body off a wall, or two off each other |
| `walk::Hopper` | A body that runs, falls under gravity, and jumps while standing, at speeds fixed here |
| `board::Board` | Where a grid of square cells sits on the screen, and which cell a point is in |
| `maze::Walls` | Digs a perfect maze by depth-first search, says whether a body may stand somewhere, and draws the walls |
| `map::Map` | The letter at a cell or under a point, and which cells a body reaches into |
| `shape::stroke_hits` | Whether a stroke touches a box, by clipping rather than sampling |
| `aim::drawn`, `aim::cast` | A drag back from an anchor, and the capped launch speed when released |
| `sky` | A fixed starfield, the ship, and shots going either way |

## The Clock

- Game time advances in ticks of a sixtieth of a second. A game never reads a wall clock.
- `advance` applies actions and lets the given ticks pass. `apply` lands one action on the tick to come, `lapse` lets ticks pass, and `step` is one tick.
- The session takes no rate; each call names its ticks.
- The session enforces `LIMIT` and reports the outcome and the game seconds.
- The clock is drawn only when `show_clock` asks.
- `session::desktop` is what is true of every game played on a screen with a mouse and a keyboard at a `Pace`: a clock, or a few ticks at each press of `session::ADVANCE`, the F1 key. `session::task` is that followed by the game's prompt.

## The Shell

`Shell` is what a person plays. It holds the game picked out, the seed of the next round, a `Session`, and which screen is up.

- `act` is handed one action. During a round it goes to the session; between rounds the shell keeps it, so a key still down when a round begins goes in held.
- `advance` is told how much of the player's time passed, which becomes game time at the shell's rate, catching up at most a tenth of a second of game time on a slow frame.
- `draw` puts the current screen in the frame and `title` says what the window is called.
- Only the menu listens to key repeats.
- The key that changed the screen does nothing else until it is released.
- Opened on the menu, escape from anywhere returns to it. Opened on one game, escape closes the shell.
- The shell reads no clock; a window and a web page drive it alike.

## The Replay

- A `Session` records every action `apply` lands, beside the tick it lands at the start of. Nothing lands once the episode is over.
- `replay::replay` plays a fresh session of the same game and seed through a record to a given tick or the end, handing a callback the screen at the start and after every tick. What landed on the last tick is applied last, and the session is handed back to compare with the original.
- The record is the recording; the frames are made afterward.
