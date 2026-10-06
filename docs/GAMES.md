# Games

One module per game under `crates/engine/src/games/`, one line per game in `catalog::CATALOG`.

## What A Game Is

```rust
pub const SPEC: Spec;
pub fn build(seed: u64) -> Box<dyn Game>;
```

`Spec` holds everything that does not change between episodes:

- `name` — The short name used on the command line.
- `title` — The title shown on screen.
- `brief` — The paragraph a player is told before the round.
- `controls` — The keys and pointer actions the game reads.
- `pointer` — Whether the pointer is used, and so drawn.
- `scoring` — `Boolean` (finishing) or `Time`.
- `varies` — Whether the seed changes the layout.
- `tags` — The tags from [TAGS.md](TAGS.md).

`Game` is two methods: `update` runs one tick and returns the `Outcome`, and `draw` puts the state on the screen, also after the round is over. A game holds its own `Rng` and reads nothing else from outside.

## Conventions Every Game Keeps

- Nothing is drawn over a game but the pointer and, with `--countdown`, the clock.
- A mistake ends the round. There are no lives. `typer` alone allows five slips.
- One key does one thing everywhere: rising is UP or SPACE, and stepping a square takes one press of an arrow.
- What is at stake is drawn as a word, a color, and a shape, not one of them.
- Nothing is random except through the game's own `Rng`.
- Trigonometry goes through `libm`, which gives the same bits on every platform; `clippy.toml` refuses the `f32` and `f64` methods that would ask the platform instead.
- Nothing moves on anything but a tick, and a game won by lasting checks for a loss before calling the round won.
- `game::LIMIT` is three minutes for every game.

## The Catalog

| Command | Title | Skill under test | Controls | Scored on | Deterministic | Description |
|---|---|---|---|---|---|---|
| `arena` | Aftershock Arena | Setting up collisions with things that are steering at you, on ice | four keys | time | yes | Knock three chasers into the live wall of an icy ring without touching it yourself. |
| `balance` | Balance Ball | Continuous control of a second-order system | two keys | time | yes | Tilt a beam to hold a rolling ball inside a band for three seconds. |
| `balloon` | Balloon Blastoff | Stopping a count at the right number against a slow leak | two keys | time | yes | Inflate a leaking balloon to a dashed circle and launch it. |
| `beam` | Beam Bounce | Reasoning about a reflection, then laying it in by eye | three keys | time | yes | Turn one mirror so the beam reflects into the far portal, five rooms. |
| `cipher` | Cipher Cinco | Reading letters and matching them against words | pointer | time | no | Click the word spelled by the loose letters, fifteen times. |
| `circuit` | Circuit Connect | Turning pieces until a path closes | pointer | time | yes | Turn sockets until the circuit closes, five boards. |
| `cleaver` | Clever Cleaver | Sweeping through moving targets, and not through others | pointer, held | finishing | no | Sweep the pointer through twenty pale blocks in thirty seconds, sparing the marked ones and missing none. |
| `circles` | Clicking Circles | Pointing at a target that is running out | pointer | time | no | Click twenty rings before they close. |
| `clocked` | Clocked In | Keeping time with nothing on the screen to keep it by | one key | time | no | Start and stop an unseen stopwatch within a tenth of the time asked, five times. |
| `census` | Con Census | Holding two running counts over half a minute, then reporting one | number keys | finishing | no | Count two kinds of drifting thing for thirty seconds, then type the count of the one asked for. |
| `conveyor` | Conveyor Collect | Picking the wanted thing out of a moving stream | pointer | time | no | Click the wanted piece as it passes on the belt, ten times. |
| `curl` | Curling Cast | Setting a body going and having no further hold on it | pointer, held | time | yes | Throw three stones to knock the red stone off the house and leave all three of yours on it. |
| `duo` | Dungeon Duo | Running two bodies on two sets of keys | six keys, held | time | yes | Bring two squares on two sets of keys together through gates that only the other can hold open, three rooms. |
| `dash` | Dynamic Dash | Reacting to what is arriving, in two different ways | two keys, one held | finishing | no | Jump the blocks and duck the bars for thirty seconds. |
| `eye` | Eerie Eye | Holding a key only while it is safe to | one key, held | time | no | Hold SPACE to walk while the eye is turned away. |
| `rocks` | Falling Rocks | Keeping clear of several warnings at once, each landing at its own moment | four keys, held | finishing | no | Stay out from under the rocks falling on a floor of sixteen squares for thirty seconds. |
| `flip` | Flip Flop Hop | Jumping to a beat | three keys, held | time | yes | Run and jump through three rooms whose red and blue blocks swap solidity every two seconds. |
| `fruits` | Floundering Fruits | Getting under what is falling | two keys, held | time | no | Move a bucket to catch twenty falling fruit. |
| `galaxy` | Galaxy Gunner | Shooting into a formation while reading the dives that leave it | three keys, two held | time | yes | Shoot down twenty drones that take turns diving at you. |
| `ghost` | Ghost Tag | Keeping room to move where the chaser ignores the walls | four keys, held | finishing | yes | Keep away for thirty seconds from a ghost that passes through the obstacles you cannot. |
| `gravity` | Go Go Gravity | Committing to a route through a room, one surface at a time | three keys | time | yes | Walk ten rooms, flipping gravity while standing to get past the spikes. |
| `glyph` | Grid Glyph | Deducing a grid from counts along its rows and columns | pointer, both buttons | time | no | Solve a 5×5 nonogram. |
| `guards` | Guard Gauntlet | Reading patrols ahead of time | four keys, held | time | yes | Cross three rooms past patrolling guards. Standing still for half a second makes you invisible. |
| `hex` | Hexagon Home | Planning a wall against something that always takes the shortest way out | pointer | time | yes | Fill hexagons to hem in a runner that always takes a shortest path to the rim, three boards. |
| `fly` | High Flyer | Keeping a rhythm of small lifts against a steady fall | one key | finishing | no | Tap for lift through twenty pillars for thirty seconds. |
| `hue` | Hue Hunt | Holding a color in mind once it is gone | pointer, held, one key | time | no | Match a color shown for five seconds on three sliders, three times. |
| `typer` | Hyper Typer | Typing exactly what is written | letter keys | time | no | Type twenty five-letter words with at most five slips. |
| `isles` | Ice Isles | Planning slides that only a rock can stop | four keys | time | yes | Slide across three isles of ice, stopping only at rocks and never off the edge. |
| `quotients` | Intel Seeker | Telling which of five things differs, and in which way | pointer | time | no | Click the one card of five that differs from the other four, fifteen times. |
| `jam` | Jam Jostle | Planning several moves ahead to clear one path through a packed lot | pointer, held | time | yes | Slide vehicles aside until the red car can leave, three lots. |
| `jump` | Jump Journey | Steering something you cannot stop from bouncing | two keys | time | no | Steer a ball that bounces off every ledge up a sinking tower. |
| `jungle` | Jungle Joyride | Timing a let-go and a catch through a swing | three keys, one held | time | yes | Swing along ten vines, holding SPACE to hold on. |
| `keys` | Kept Key | Following one thing among eight alike | pointer | finishing | no | Follow one of eight matching keys through half a minute of shuffling, then click it. |
| `lights` | Last Lights | Working out which squares to click when every click also turns over its neighbors | pointer | time | no | Turn every square black, where a click toggles a square and its four neighbors, three boards. |
| `latin` | Latin Square | Narrowing down what can go where from what is already written | pointer, number keys | time | no | Fill a 6×6 Latin square with eighteen blanks. |
| `maze2` | Marauder Maze | Routing around something that routes straight at you | four keys | time | no | Cross a looping maze while a slower block hunts you along the shortest path. |
| `maze4` | Mega Maze | Keeping a map in mind of a place too big to see at once | four keys | time | no | Cross a maze five screens wide that scrolls around you, from the middle to the farthest cell. |
| `mimic` | Memory Mimic | Holding a growing sequence in mind and giving it back | pointer | time | no | Repeat a sequence of lit panels that grows from two to six. |
| `mines` | Mine Sweep | Reading a constraint off numbers and acting only where it is settled | pointer | time | no | Minesweeper with ten mines in a hundred squares, dealt so that no guess is ever needed. |
| `maze3` | Murky Maze | Building a map of a place you can only see a room of at a time | four keys | time | no | Find three keys in a maze that shows only what you have a clear line of sight to. |
| `maze` | Mythic Maze | Reading a layout off the screen and following a route | four keys | time | no | Walk from the top left to the bottom right of a maze. |
| `paddle` | Paddle Power | Putting something under a falling ball, and sending it where you want it next | two keys, held | finishing | yes | Keep a ball bouncing on a paddle for thirty seconds, steering it by the paddle's tilt. |
| `paint` | Paint Path | Finding a way around a board that leaves nothing out | four keys | time | yes | Step on every square of a board once, three boards. |
| `pairs` | Perfect Pairs | Remembering what was under a tile that is face down again | pointer | time | no | Match eight pairs of face-down tiles. |
| `pack` | Pieces Packing | Turning shapes in the mind's eye until a set of them fits a space | pointer, both buttons, held | time | yes | Drag and turn pieces to fill a 6×6 grid, three grids. |
| `pallets` | Push Pallets | Planning pushes that cannot be taken back | four keys | time | yes | Push every crate onto a mark, three rooms. A crate stuck in a corner leaves the round to run out. |
| `pyro` | Pyro Passage | Steering a body against a steady fall through gaps that fire sweeps | three keys, held | time | yes | Fly a jetpack through three caves past turning bars of fire. |
| `quiz` | Quantitative Quiz | Arithmetic over a board, and searching it | pointer | time | no | Click the pair of nine numbers that adds to the total, five times. |
| `race` | Raceway Royale | Acting on a heading rather than on the screen's own left and right | three keys | time | yes | Drive one lap with a throttle and a car that turns about itself. |
| `reflexes` | Real Reflexes | Reacting the moment something changes, and not before | pointer | time | no | Click within thirty frames of the panel turning green, five times. |
| `rings` | Rolling Rings | Working out where a ball will roll as the whole maze turns under it | two keys, held | time | yes | Turn a round maze of five rings until the ball in the middle rolls out. |
| `shuttle` | Shuttle Shootout | Lining up shots on a moving row while dodging fire aimed where you are heading | three keys, two held | time | yes | Shoot down a row of ten saucers that sweeps closer and fires at where you are heading. |
| `ski` | Slalom Station | Lining up a body that slides on after every push | four keys, held | time | no | Slide on ice through six numbered gates in order, the way each arrow points. |
| `sword` | Slanted Sword | Dragging through a target in the one direction it asks for | pointer, held | time | no | Drag through ten blocks the way each block's arrow points. |
| `slide` | Slide Sixteen | Planning a route for one piece through pieces that block it | pointer | time | yes | Fifteen-puzzle: get tile 1 to the bottom right. |
| `sling` | Slingshot Sniper | Reading an arc off a drawn-back sling before letting go of it | pointer, held | time | yes | Drag and release a sling to drop the ball into a hanging bucket, one shot at each of five. |
| `slots` | Slot Star | Stopping something moving on the one place it has to stop | one key | time | no | Stop three reels with SPACE so a row matches. |
| `setdown` | Soft Setdown | Bleeding off speed against a limit before touching down | three keys | time | no | Land a falling craft softly on the pad, five times. |
| `spire` | Spire Stack | Planning a sequence of moves before making them | pointer | time | yes | Towers of Hanoi with four rings. |
| `stones` | Stone Standoff | Finding the one move that leaves an opponent nothing | pointer | time | yes | Nim against an opponent that never misses a win, three rounds. |
| `serpent` | Super Serpent | Steering under a fixed cadence without trapping yourself | four keys | time | no | Snake, ten pellets. |
| `targets` | Target Practice | Searching a board through a small window and hitting what it finds | pointer | time | no | Sweep a scope over a board to find and click fifteen marks. |
| `tiles` | Tile Tech | Striking and holding four lanes in time | four keys, held | finishing | no | Strike and hold four lanes of falling tiles for thirty seconds. |
| `traffic` | Traffic Trot | Timing a crossing through gaps that are moving | four keys | time | no | Cross four lanes of quickening traffic, five times. |
| `tunnel` | Tunnel Tracker | Following a line out of sight and saying where it comes out | pointer | time | no | Click the mouth a marker will come out of before it does, ten times. |
| `twins` | Twin Trace | Filling a board with routes that must not cross | pointer, held | time | yes | Join each pair of colored dots with non-crossing lines that cover the board, three boards. |
| `caverns` | Underwater Caverns | Planning a way through by where the next breath is | four keys, held | time | yes | Swim three flooded caverns on a breath at a time, refilling at air pockets in the ceiling. |
| `volley` | Very Volley | Tracking and anticipating something faster than you can move | two keys | finishing | no | Keep a quickening ball out of the wall behind you for thirty seconds. |
| `wire` | Wire Wind | Holding a hand steady over a long stretch | pointer, held | time | no | Drag a dot the length of a wire maze without touching the wire. The dot follows the pointer at a fixed pace. |
| `search` | Word Search | Searching a grid of letters for a word that may run any way | pointer, held | time | no | Find five words in a 10×10 letter grid, in any of eight directions. |

## Words

`cipher`, `typer`, and `search` share `words::WORDS`: five-letter words, no two anagrams and none a palindrome.

## Adding One

1. Write the module under `games/` and its tests under `tests/games/`.
2. Name it in `games/mod.rs` and add its line to `catalog::CATALOG`, which is kept in title order.
3. Run `cargo test`. The catalog tests play every game badly to its limit and check that a seed and a script decide the picture exactly.
4. Add its row to the table above and to [TAGS.md](TAGS.md).

`tests/playing.rs` holds what a test needs to play a game: a look, held keys, and `fewest`, a search for the fewest looks out of a place.
