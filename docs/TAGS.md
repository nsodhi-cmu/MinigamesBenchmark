# Tags

Every game carries tags that help categorize the games. You can filter on the tags to see what types of minigames your agent excels at.

## Time Based Tags

These tags describe how the round is settled from a time perspective. This set is disjoint and exhaustive. Each game has exactly one of them.

- **`fixed`** — The round lasts thirty seconds unless the player loses first. Either the player wins after thirty seconds of survival, or, after thirty seconds, the player makes one final decision and then the round ends. These games should be treated as pass-fail rather than evaluated for how long an agent takes to complete.
- **`bounded`** — Settled on time. The theoretical fastest time is not zero because there is some property inherent to the game that takes time, such as physics.
- **`unbounded`** — Settled on time. The theoretical fastest time an agent can complete the game is just a few frames. This idea can be formalized by saying that as the tick rate of the game approaches infinity, the theoretical fastest time approaches zero.

| Tag | Games | Which |
|---|---|---|
| `fixed` | 10 | `census` `cleaver` `dash` `fly` `ghost` `keys` `paddle` `rocks` `tiles` `volley` |
| `bounded` | 38 | `arena` `balance` `beam` `caverns` `clocked` `conveyor` `curl` `duo` `eye` `flip` `fruits` `galaxy` `gravity` `guards` `hue` `isles` `jump` `jungle` `maze` `maze2` `maze3` `maze4` `mimic` `pyro` `race` `reflexes` `rings` `serpent` `setdown` `shuttle` `ski` `sling` `slots` `sword` `traffic` `tunnel` `typer` `wire` |
| `unbounded` | 22 | `balloon` `cipher` `circles` `circuit` `glyph` `hex` `jam` `latin` `lights` `mines` `pack` `paint` `pairs` `pallets` `quiz` `quotients` `search` `slide` `spire` `stones` `targets` `twins` |

## Fail Condition Flags

These tags describe how the player can fail. This set is disjoint and exhaustive. Each game has exactly one of them. The term *idle* is used below to mean the player not making any inputs. These definitions ignore possible loopholes. (For example, it may be possible to idle and win a game classed as lethal if the idling is for the final second.)

- **`lethal`** — There does not exist any state where the player can safely idle.
- **`hostile`** — There exists at least one state where the player cannot safely idle, and at least one state where the player can safely idle.
- **`strict`** — A player can always safely idle, but it is possible to fail the game from a wrong input.
- **`friendly`** — The only way to fail the game is to run out of time.

| Tag | Games | Which |
|---|---|---|
| `lethal` | 18 | `arena` `circles` `cleaver` `dash` `fly` `galaxy` `ghost` `jump` `maze2` `paddle` `reflexes` `rocks` `serpent` `setdown` `shuttle` `tiles` `tunnel` `volley` |
| `hostile` | 10 | `balance` `caverns` `flip` `gravity` `guards` `jungle` `pyro` `ski` `traffic` `wire` |
| `strict` | 23 | `balloon` `beam` `census` `cipher` `clocked` `conveyor` `curl` `duo` `eye` `hex` `hue` `isles` `keys` `mimic` `mines` `paint` `quiz` `quotients` `sling` `slots` `stones` `sword` `typer` |
| `friendly` | 19 | `circuit` `fruits` `glyph` `jam` `latin` `lights` `maze` `maze3` `maze4` `pack` `pairs` `pallets` `race` `rings` `search` `slide` `spire` `targets` `twins` |

## Descriptive Tags

These tags describe whichever game they fit.

- **`puzzle`** — A puzzle where the challenge comes from working out the solution and acting is easy once you have it.
- **`quiz`** — A game made up of asking the player questions.
- **`language`** — Words, language, or character recognition are integral to the game.
- **`memory`** — Information must be carried across time because the game stops showing it.
- **`aim`** — The game requires putting the pointer somewhere precisely.
- **`timing`** — The game requires acting at/for a moment rather than in a place: when it turns, as it passes, and/or before it closes.
- **`physics`** — The game requires some concept of physics and/or mechanics.
- **`navigate`** — The game requires navigating through a place.
- **`explore`** — The screen does not show all information needed to complete the game. The game must be explored to reveal all the information needed to complete the game. A game whose later "rooms" are out of sight is not this.
- **`overhead`** — The game is portrayed from a top-down view.
- **`platformer`** — The game is a platformer portrayed from a sideways view.
- **`grid`** — The game is played on a grid of cells.
- **`drag`** — The game requires dragging: holding the pointer down while moving it.
- **`slippery`** — The player keeps moving after inputs stop.

| Tag | Games | Which |
|---|---|---|
| `puzzle` | 18 | `beam` `circuit` `duo` `glyph` `hex` `isles` `jam` `latin` `lights` `mines` `pack` `paint` `pallets` `rings` `slide` `spire` `stones` `twins` |
| `quiz` | 5 | `census` `cipher` `quiz` `quotients` `tunnel` |
| `language` | 3 | `cipher` `search` `typer` |
| `memory` | 9 | `census` `clocked` `hue` `keys` `maze3` `maze4` `mimic` `pairs` `ski` |
| `aim` | 8 | `circles` `cleaver` `conveyor` `curl` `sling` `sword` `targets` `wire` |
| `timing` | 24 | `balloon` `caverns` `circles` `clocked` `conveyor` `dash` `eye` `flip` `fly` `fruits` `galaxy` `gravity` `guards` `jungle` `paddle` `pyro` `reflexes` `rocks` `shuttle` `slots` `tiles` `traffic` `tunnel` `volley` |
| `physics` | 16 | `arena` `balance` `caverns` `curl` `fly` `gravity` `jump` `jungle` `paddle` `pyro` `race` `rings` `setdown` `ski` `sling` `volley` |
| `navigate` | 17 | `caverns` `duo` `flip` `ghost` `gravity` `guards` `isles` `maze` `maze2` `maze3` `maze4` `pyro` `race` `serpent` `ski` `traffic` `wire` |
| `explore` | 4 | `maze3` `mines` `pairs` `targets` |
| `overhead` | 16 | `arena` `curl` `ghost` `guards` `isles` `maze` `maze2` `maze3` `maze4` `paint` `pallets` `race` `rocks` `serpent` `ski` `traffic` |
| `platformer` | 4 | `duo` `flip` `gravity` `jump` |
| `grid` | 17 | `circuit` `glyph` `hex` `isles` `jam` `latin` `lights` `mines` `pack` `paint` `pairs` `pallets` `rocks` `search` `serpent` `slide` `twins` |
| `drag` | 10 | `cleaver` `curl` `hue` `jam` `pack` `search` `sling` `sword` `twins` `wire` |
| `slippery` | 4 | `arena` `isles` `race` `ski` |

## Every Game

| Command | On screen | Tags |
|---|---|---|
| `arena` | Aftershock Arena | `bounded` `lethal` `physics` `overhead` `slippery` |
| `balance` | Balance Ball | `bounded` `hostile` `physics` |
| `balloon` | Balloon Blastoff | `unbounded` `strict` `timing` |
| `beam` | Beam Bounce | `bounded` `strict` `puzzle` |
| `cipher` | Cipher Cinco | `unbounded` `strict` `quiz` `language` |
| `circuit` | Circuit Connect | `unbounded` `friendly` `puzzle` `grid` |
| `cleaver` | Clever Cleaver | `fixed` `lethal` `aim` `drag` |
| `circles` | Clicking Circles | `unbounded` `lethal` `aim` `timing` |
| `clocked` | Clocked In | `bounded` `strict` `timing` `memory` |
| `census` | Con Census | `fixed` `strict` `memory` `quiz` |
| `conveyor` | Conveyor Collect | `bounded` `strict` `aim` `timing` |
| `curl` | Curling Cast | `bounded` `strict` `aim` `physics` `overhead` `drag` |
| `duo` | Dungeon Duo | `bounded` `strict` `puzzle` `navigate` `platformer` |
| `dash` | Dynamic Dash | `fixed` `lethal` `timing` |
| `eye` | Eerie Eye | `bounded` `strict` `timing` |
| `rocks` | Falling Rocks | `fixed` `lethal` `timing` `overhead` `grid` |
| `flip` | Flip Flop Hop | `bounded` `hostile` `navigate` `timing` `platformer` |
| `fruits` | Floundering Fruits | `bounded` `friendly` `timing` |
| `galaxy` | Galaxy Gunner | `bounded` `lethal` `timing` |
| `ghost` | Ghost Tag | `fixed` `lethal` `navigate` `overhead` |
| `gravity` | Go Go Gravity | `bounded` `hostile` `navigate` `timing` `platformer` `physics` |
| `glyph` | Grid Glyph | `unbounded` `friendly` `puzzle` `grid` |
| `guards` | Guard Gauntlet | `bounded` `hostile` `navigate` `timing` `overhead` |
| `hex` | Hexagon Home | `unbounded` `strict` `puzzle` `grid` |
| `fly` | High Flyer | `fixed` `lethal` `timing` `physics` |
| `hue` | Hue Hunt | `bounded` `strict` `memory` `drag` |
| `typer` | Hyper Typer | `bounded` `strict` `language` |
| `isles` | Ice Isles | `bounded` `strict` `puzzle` `navigate` `overhead` `grid` `slippery` |
| `quotients` | Intel Seeker | `unbounded` `strict` `quiz` |
| `jam` | Jam Jostle | `unbounded` `friendly` `puzzle` `grid` `drag` |
| `jump` | Jump Journey | `bounded` `lethal` `physics` `platformer` |
| `jungle` | Jungle Joyride | `bounded` `hostile` `physics` `timing` |
| `keys` | Kept Key | `fixed` `strict` `memory` |
| `lights` | Last Lights | `unbounded` `friendly` `puzzle` `grid` |
| `latin` | Latin Square | `unbounded` `friendly` `puzzle` `grid` |
| `maze2` | Marauder Maze | `bounded` `lethal` `navigate` `overhead` |
| `maze4` | Mega Maze | `bounded` `friendly` `navigate` `memory` `overhead` |
| `mimic` | Memory Mimic | `bounded` `strict` `memory` |
| `mines` | Mine Sweep | `unbounded` `strict` `puzzle` `explore` `grid` |
| `maze3` | Murky Maze | `bounded` `friendly` `navigate` `explore` `overhead` `memory` |
| `maze` | Mythic Maze | `bounded` `friendly` `navigate` `overhead` |
| `paddle` | Paddle Power | `fixed` `lethal` `timing` `physics` |
| `paint` | Paint Path | `unbounded` `strict` `puzzle` `overhead` `grid` |
| `pairs` | Perfect Pairs | `unbounded` `friendly` `memory` `explore` `grid` |
| `pack` | Pieces Packing | `unbounded` `friendly` `puzzle` `grid` `drag` |
| `pallets` | Push Pallets | `unbounded` `friendly` `puzzle` `overhead` `grid` |
| `pyro` | Pyro Passage | `bounded` `hostile` `navigate` `timing` `physics` |
| `quiz` | Quantitative Quiz | `unbounded` `strict` `quiz` |
| `race` | Raceway Royale | `bounded` `friendly` `navigate` `physics` `overhead` `slippery` |
| `reflexes` | Real Reflexes | `bounded` `lethal` `timing` |
| `rings` | Rolling Rings | `bounded` `friendly` `puzzle` `physics` |
| `shuttle` | Shuttle Shootout | `bounded` `lethal` `timing` |
| `ski` | Slalom Station | `bounded` `hostile` `physics` `navigate` `overhead` `memory` `slippery` |
| `sword` | Slanted Sword | `bounded` `strict` `aim` `drag` |
| `slide` | Slide Sixteen | `unbounded` `friendly` `puzzle` `grid` |
| `sling` | Slingshot Sniper | `bounded` `strict` `aim` `physics` `drag` |
| `slots` | Slot Star | `bounded` `strict` `timing` |
| `setdown` | Soft Setdown | `bounded` `lethal` `physics` |
| `spire` | Spire Stack | `unbounded` `friendly` `puzzle` |
| `stones` | Stone Standoff | `unbounded` `strict` `puzzle` |
| `serpent` | Super Serpent | `bounded` `lethal` `navigate` `overhead` `grid` |
| `targets` | Target Practice | `unbounded` `friendly` `aim` `explore` |
| `tiles` | Tile Tech | `fixed` `lethal` `timing` |
| `traffic` | Traffic Trot | `bounded` `hostile` `navigate` `timing` `overhead` |
| `tunnel` | Tunnel Tracker | `bounded` `lethal` `quiz` `timing` |
| `twins` | Twin Trace | `unbounded` `friendly` `puzzle` `grid` `drag` |
| `caverns` | Underwater Caverns | `bounded` `hostile` `navigate` `timing` `physics` |
| `volley` | Very Volley | `fixed` `lethal` `timing` `physics` |
| `wire` | Wire Wind | `bounded` `hostile` `navigate` `aim` `drag` |
| `search` | Word Search | `unbounded` `friendly` `language` `grid` `drag` |
