<p align="center">
  <a href="https://minigames.sodhi.tech"><img src="https://minigames.sodhi.tech/assets/og.png" alt="Minigames Benchmark: short games that computer-use agents must finish from pictures alone" width="760"></a>
</p>

<p align="center">
  <a href="https://play.minigames.sodhi.tech"><b>Play in your browser</b></a>
  &nbsp;·&nbsp;
  <a href="https://minigames.sodhi.tech">Website</a>
  &nbsp;·&nbsp;
  <a href="docs/">Documentation</a>
</p>

Small games that a person can finish in about a minute, and that an agent has to finish from pictures alone.

Every game is a 640×480 picture and a handful of keys. An agent is given the frame and nothing else, and sends back what a mouse and a keyboard can do. Game time is counted in ticks, and the [pace](#playing-an-agent) decides when a tick passes.

## Playing It Yourself

The quickest way is [in your browser](https://play.minigames.sodhi.tech). To play on the desktop, build the `minigame` command with Rust ([Building](#building)):

```
cargo install --path crates/cli
minigame                      # the menu: pick a game and play it
minigame play serpent         # or go straight to one
minigame list                 # the catalog, as text
minigame prompt serpent       # what a player is told before it starts
minigame shot serpent --after 90 --out serpent.png
```

The menu moves on the arrow keys and opens a game on enter. Enter plays again on the next layout; escape goes back to the menu.

```
minigame play --seed 0             # the menu, opening on layout 0
minigame play serpent --countdown  # count the limit down on screen
minigame play maze --seed 0        # that maze, straight away
minigame play race --clock 30      # half speed, everything easier
```

The same games are a Python module, once `uv sync` has built it:

```python
import minigames

episode = minigames.Episode("circles", seed=0)
print(episode.prompt)
while episode.outcome == "playing":
    picture = episode.frame()          # a PNG, and nothing else
    episode.act("click 320 240", ticks=6)
print(episode.report())
```

## The Games

<p align="center">
  <img src="https://minigames.sodhi.tech/assets/gameplay.gif" alt="Twelve of the games, played: a raceway, hexagons, twin lines, traffic, an icy arena, flooded caverns, falling tiles, a word search, a round maze, a saucer row, ice isles, and a packing grid" width="760">
</p>

Every game has the same three-minute limit. A run is scored one per episode finished inside it; time is reported beside the score, not folded into it. [GAMES.md](docs/GAMES.md) lists every game and what it measures, and [TAGS.md](docs/TAGS.md) groups them.

## Playing An Agent

An agent plays in a container of its own, on a 640×480 display that shows the game and nothing else. It takes screenshots and works the mouse and keyboard, as on a Linux desktop. The game plays in a second container that carries the display, an X server of its own, so the host's display, whether X11, Wayland, or none, plays no part.

- **On a clock**, with `--clock N`, the game does not wait: it runs `N` ticks of game time per second of the agent's, 60 being full speed and `--clock 6` ten times slower, limit included.
- **In ticks**, with `--ticks N`, the game stands still, and each press of F1 lets `N` ticks pass, one where `N` is left out. An agent must hold a key or button down across a press of F1: one pressed and let go between two presses lands nothing but where the pointer went. `--idle-timeout` and `--total-timeout` stop an episode whose agent stops pressing F1 or runs too long; neither applies unless given.
- The agent's command gets the task, the screen, the pace, and the game's prompt, in place of `{task}`; `podman run --rm minigames-game task maze --ticks 3` prints it.

```
podman build -t minigames-game .                   # the image a game is played in
podman build -t minigames-agent agent              # the image an agent plays from
python -m minigames.bench --run-agent minigames-agent \
    "harness --base-url http://host.containers.internal:PORT/v1 --model MODEL clock {task}" \
    --engine podman --clock 6
```

A run is written into `runs/`, or wherever `--out` says, and plays seed 0 unless `--seed` names another. Every flag is listed in [BENCH.md](docs/BENCH.md#the-command-line). The benchmark itself is every game at four paces, `--ticks 3`, `--clock 6`, `--clock 20`, and `--clock 60`, set out with how long each takes in [BENCH.md](docs/BENCH.md#the-benchmark).

The agent image comes with `harness`, a deliberately minimal placeholder for your own agent. Each turn it screenshots, asks any OpenAI-compatible endpoint what to do, and runs the `pyautogui` code that comes back, remembering nothing. `harness clock` plays in real time; `harness ticks` presses F1 after each answer. The endpoint and model are `--base-url` and `--model`, or `OPENAI_BASE_URL` and `OPENAI_MODEL`; a key is `OPENAI_API_KEY`.

### Bringing Your Own Agent

1. Write a Containerfile of your own, in a folder of its own, that installs your agent, starting `FROM minigames-agent` for Python, `pyautogui`, `xdotool`, and `x11-utils`, or from any Linux image that has `sh` and `sleep`, and build it under a name of your own. Environment settings go in as `ENV` lines, or before the command as `env NAME=value`. The command is written into the run's records as it is given, so keep secrets such as API keys out of it and in the image instead.
2. Run it with the command that starts it, `{task}` standing for the task:

```
podman build -t my-agent my-agent
python -m minigames.bench --run-agent my-agent "my-agent {task}" --engine podman
```

- `--engine` is `podman` or `docker`, with no default. Each engine keeps its own images, built with the engine that plays them; docker wants the file named: `docker build -f agent/Containerfile -t minigames-agent agent`.
- The agent's container has `DISPLAY=:0`, `WIDTH=640`, `HEIGHT=480`, and its engine's default network. A server on this machine, such as a local model, is `host.containers.internal` under podman and `host.docker.internal` under docker, and must listen on more than `127.0.0.1`.
- Nothing stops an agent from opening windows over the game, closing the game's window, or installing software as it plays, but a score won any way other than by playing the game as intended does not count.

## Why Rust

The games are Rust and the runner is Python. The games are Rust for four things:

- **Determinism.** An episode is its seed and the inputs that landed, and it plays out the same bits every time: every episode is checked by playing it again, and every recording is made that way. There is no garbage collector or runtime between the game and its arithmetic, and trigonometry goes through `libm` rather than the platform's own math library.
- **One codebase, every target.** The same game code is the `minigame` binary in the game's container, the Python extension the runner replays with, the WebAssembly build that plays in a browser, and the desktop game a person plays.
- **Speed.** Every frame is drawn in software, sixty times a second of game time, with no GPU. Many episodes can play at once with `--jobs`, and a test or a recording plays thousands of ticks in moments.
- **A small game image.** The display client is pure Rust, so the game's container holds one compiled binary and the X server, and nothing to install beside them.

## The Components

| Component | What it describes |
|---|---|
| [engine](docs/ENGINE.md) | The picture, the action language, the clock, and the episode |
| [cli](docs/CLI.md) | The `minigame` command and its window |
| [games](docs/GAMES.md) | Every game, and what it measures |
| [tags](docs/TAGS.md) | The tags, and which games carry them |
| [bench](docs/BENCH.md) | Running agents in containers, the harness, and scoring |
| [web](docs/WEB.md) | The games in a browser |

## Building

```
cargo build --release                       # target/release/minigame
cargo install --path crates/cli             # puts minigame on PATH
```

Building needs Rust and a C compiler. On Linux the window loads the desktop's own X11 or Wayland libraries, and `libxkbcommon`, when it opens.

The same games play in a browser:

```
rustup target add wasm32-unknown-unknown     # once
cargo build --release -p minigame-web --target wasm32-unknown-unknown
mkdir -p target/site
cp crates/web/index.html target/wasm32-unknown-unknown/release/minigame_web.wasm target/site
python3 -m http.server -d target/site        # then open http://localhost:8000
```

```
uv sync --extra bench                       # also builds the extension
uv run maturin develop --release            # rebuild after changing Rust
```

```
cargo test                                  # optimized, with overflow checks kept on
uv run pytest tests                         # the Python side
uv run --extra bench pytest tests --containers podman   # and episodes in containers, building their images
cargo fmt --all --check                     # before a commit
cargo clippy --all-targets -- -D warnings   # and this
```

[`.github/workflows/ci.yml`](.github/workflows/ci.yml) runs these on Linux, with docker, for every push to `main` and every pull request. Nothing is tested on macOS or Windows.

## Conventions

[`AGENTS.md`](AGENTS.md) governs how code here is written. Documentation lives under [`docs/`](docs/).

## License

Apache 2.0. See [LICENSE](LICENSE).
