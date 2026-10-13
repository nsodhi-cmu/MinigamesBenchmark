<p align="center">
  <a href="https://minigames.sodhi.tech"><img src="https://minigames.sodhi.tech/assets/og.png" alt="Minigames Benchmark: short games that computer-use agents must finish from pictures alone" width="760"></a>
</p>

<p align="center">
  <a href="https://play.minigames.sodhi.tech"><b>Play in your browser</b></a>
  &nbsp;·&nbsp;
  <a href="https://minigames.sodhi.tech">Website</a>
  &nbsp;·&nbsp;
  <a href="https://minigames.sodhi.tech/#games">Games</a>
  &nbsp;·&nbsp;
  <a href="https://minigames.sodhi.tech/#method">Paces</a>
  &nbsp;·&nbsp;
  <a href="https://minigames.sodhi.tech/#run">Run Your Agent</a>
  &nbsp;·&nbsp;
  <a href="https://minigames.sodhi.tech/#faq">FAQ</a>
  &nbsp;·&nbsp;
  <a href="docs/">Documentation</a>
</p>

A compendium of minigames that humans can complete relatively easily in about a minute, but that state-of-the-art agents really struggle with.

## Playing It Yourself

The quickest way is [in your browser](https://play.minigames.sodhi.tech), or [at an agent's pace](https://minigames.sodhi.tech/#play). To play on the desktop, build the `minigame` command with Rust ([Building](#building)):

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
    picture = episode.frame()          # a PNG
    episode.act("click 320 240", ticks=6)
print(episode.report())
```

## The Games

<p align="center">
  <img src="https://minigames.sodhi.tech/assets/gameplay.gif" alt="Twelve of the games, played: a raceway, hexagons, twin lines, traffic, an icy arena, flooded caverns, falling tiles, a word search, a round maze, a saucer row, ice isles, and a packing grid" width="760">
</p>

Every game has the same three-minute limit. A run scores one for every game finished inside it; time is reported separately. [GAMES.md](docs/GAMES.md) lists every game and what it measures, [TAGS.md](docs/TAGS.md) groups them, and the [website](https://minigames.sodhi.tech/#games) has a clip of each.

## Playing An Agent

An agent plays in [its own container](docs/BENCH.md#an-agent-in-a-container), on a 640×480 display that shows only the game. It takes screenshots and works the mouse and keyboard, as on a Linux desktop. The game plays in a second container that carries the display on its own X server, so the host's display, whether X11, Wayland, or none, plays no part.

- **On a clock**, with `--clock N`, the game does not wait: it runs `N` ticks of game time per second of real time, 60 being full speed and `--clock 6` ten times slower, limit included.
- **In ticks**, with `--ticks N`, the game stands still, and each press of F1 lets `N` ticks pass, one if `N` is left out. A key or button must be held down across a press of F1: one pressed and released between two presses only moves the pointer. `--idle-timeout` and `--total-timeout` stop an episode whose agent stops pressing F1 or runs too long; both are off by default.
- `{task}` in the agent's command becomes the task: the screen, the pace, and the game's prompt. `podman run --rm minigames-game task maze --ticks 3` prints it.

```
podman build -t minigames-game .                   # the image a game is played in
podman build -t minigames-agent agent              # the image an agent plays from
uv run python -m minigames.bench --run-agent minigames-agent \
    "harness --base-url http://host.containers.internal:PORT/v1 --model MODEL clock {task}" \
    --engine podman --clock 6
```

A run is written into `runs/`, or wherever `--out` says, as a [`result.json` and an `inputs.txt`](docs/BENCH.md#what-a-run-writes) for each game, and plays seed 0 unless `--seed` names another. Every episode is checked by [replaying its inputs](docs/ENGINE.md#the-replay). [BENCH.md](docs/BENCH.md#the-command-line) lists every flag, among them `--games`, `--tags`, `--jobs`, `--record`, `--resume`, and `--report`. The benchmark is every game at four paces, `--ticks 3`, `--clock 6`, `--clock 20`, and `--clock 60`; [BENCH.md](docs/BENCH.md#the-benchmark) and the [website](https://minigames.sodhi.tech/#method) show how long each takes.

The agent image comes with [`harness`](docs/BENCH.md#the-harness-in-the-agent-image), a deliberately minimal placeholder for your own agent. Each turn it screenshots, asks any OpenAI-compatible endpoint what to do, and runs the `pyautogui` code that comes back, remembering nothing. `harness clock` plays in real time; `harness ticks` presses F1 after each answer. The endpoint and model are `--base-url` and `--model`, or `OPENAI_BASE_URL` and `OPENAI_MODEL`; the key is `OPENAI_API_KEY`. The website has [its results](https://minigames.sodhi.tech/#run).

### Bringing Your Own Agent

1. Write a Containerfile, in its own folder, that installs your agent. Start `FROM minigames-agent` for Python, `pyautogui`, `xdotool`, and `x11-utils`, or from any Linux image that has `sh` and `sleep`. Environment settings go in as `ENV` lines, or before the command as `env NAME=value`. The command is saved with the run as given, so keep secrets such as API keys in the image instead.
2. Build it, and run it with the command that starts it, `{task}` standing for the task:

```
podman build -t my-agent my-agent
uv run python -m minigames.bench --run-agent my-agent "my-agent {task}" --engine podman
```

- `--engine` is `podman` or `docker`, with no default. Each engine keeps its own images, so build with the one that plays them; docker needs the file named: `docker build -f agent/Containerfile -t minigames-agent agent`.
- The agent's container has `DISPLAY=:0`, `WIDTH=640`, `HEIGHT=480`, and its engine's default network. A server on the host, such as a local model, is `host.containers.internal` under podman and `host.docker.internal` under docker, and must listen on more than `127.0.0.1`.
- Nothing stops an agent from opening windows over the game, closing the game's window, or installing software as it plays, but a score won any way other than by playing the game as intended does not count.

## The Components

| Component | Path | What it describes |
|---|---|---|
| [engine](docs/ENGINE.md) | [`crates/engine`](crates/engine) | The picture, the action language, the clock, and the episode |
| [cli](docs/CLI.md) | [`crates/cli`](crates/cli) | The `minigame` command and its window |
| [games](docs/GAMES.md) | [`crates/engine/src/games`](crates/engine/src/games) | Every game, and what it measures |
| [tags](docs/TAGS.md) | [`crates/engine/src/game.rs`](crates/engine/src/game.rs) | The tags, and which games carry them |
| [bench](docs/BENCH.md) | [`python/minigames`](python/minigames), [`crates/py`](crates/py), [`agent`](agent) | Running agents in containers, the harness, and scoring |
| [web](docs/WEB.md) | [`crates/web`](crates/web) | The games in a browser |

## Building

```
cargo build --release                       # target/release/minigame
cargo install --path crates/cli             # puts minigame on PATH
```

Building needs Rust, a C compiler, and [uv](https://docs.astral.sh/uv/); playing an agent needs podman or docker. On Linux the window loads the desktop's own X11 or Wayland libraries, and `libxkbcommon`, when it opens.

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
