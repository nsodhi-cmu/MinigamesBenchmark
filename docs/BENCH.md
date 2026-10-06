# Bench

The Python side: the module, the container runs, and their scoring.

## The Module

`crates/py` is a `pyo3` extension built by `maturin` as `minigames._core` and re-exported by `python/minigames/__init__.py`.

```python
minigames.games()      # every spec, one dictionary each, prompt included
minigames.size()       # (640, 480)
minigames.tick_hz()    # 60
minigames.replay(game, seed, record, end, each)  # a record played again: its outcome and last tick

episode = minigames.Episode(game, seed=0)   # a game and a seed, and nothing else
episode.game, episode.seed, episode.prompt   # what it is, and what a player is told
episode.outcome, episode.seconds             # "playing", "won", or "lost", and game time
episode.frame()        # the screen as a PNG
episode.pixels()       # the screen as width * height * 3 bytes
episode.act(*lines, ticks=1)   # actions land, that much game time passes; the outcome
episode.tick           # ticks of game time so far
episode.replay(each)   # every tick's pixels to `each`
episode.report()       # game, seed, outcome, ticks
```

Each `act` names the ticks it lets pass, and refuses a line it does not recognize.

## The Runner

The package `python/minigames/bench/`:

| Module | What is in it |
|---|---|
| `containers` | `play` for one episode with an agent in a container of its own, `sweep` for a run of them |
| `runner` | `chosen` and `tagged` for what a run plays, `encode` for a recording |
| `scoring` | `summarize`, which sorts a run's games into passed, failed, and incomplete |
| `__main__` | The command line |

## An Agent In A Container

`containers.play(engine, image, template, game, seed, pace, folder, record, ..., idle_timeout, total_timeout)` plays one episode with an agent in a container of its own, from `image`, that sees a display and works its mouse and keyboard. `pace` is `("clock", rate)` or `("ticks", n)`, each from 1 to 60.

- `engine` is `podman` or `docker`, always named. Every container starts with `--pull never`, so an image not yet built is refused rather than fetched.
- The game plays in a `minigames-game` container with no network, running `minigame screen` ([CLI.md](CLI.md#the-screen)). The agent's container has its engine's default network, `DISPLAY=:0`, and the screen's size as `WIDTH=640` and `HEIGHT=480`. Under docker it is also told `host.docker.internal` is this machine.
- The two share only a volume at `/tmp/.X11-unix`, which holds the display's socket. Both run with `--security-opt label=disable` and `--security-opt no-new-privileges`.
- The agent's container starts idle on `sleep infinity`. Its command is started through `sh`, which prints `minigames-gate` and waits for a line. Once the game says `ready` and the gate is open, `go` goes to the game and a line to the gate together.
- The task is `minigame task` as the game image prints it, put whole in place of every `{task}`. The command is split into words once and never parsed by a shell; one with no `{task}` is refused. A command that wants a shell can be `sh -c '... "$0"' {task}`, which hands the task to the shell as `$0`.
- On a clock the game runs to its own end whatever the agent does. In ticks it moves only on F1 and says `advanced` after each press, and it also ends, as `unfinished`, a second after the agent's command does.
- In ticks, `idle_timeout` ends an episode once that many seconds pass without a press of F1, and `total_timeout` that many seconds after `go`; the game is told to stop on standard input. Neither applies unless given, and both are refused on a clock.
- Then both containers and the volume are killed and removed. They carry the label `minigames.episode`, and everything a run starts carries `minigames.run`.
- Once over, the episode is played again from its record by `minigames.replay`, and one that comes out differently is refused.
- A container not up within a minute ends the episode with an error.

| File in `folder` | What it holds |
|---|---|
| `result.json` | `game` and `seed`; `outcome`, `won`, `lost`, or `unfinished`; `ticks` of game time; `seconds` of wall time from `go` to the end; and `stopped`, what ended an unfinished episode, `agent-exited`, `idle-timeout`, or `total-timeout`, or `null` |
| `inputs.txt` | Every action that landed, as its tick and a line of the action language, and last `over`, the outcome, and the tick |
| `agent-stdout.txt`, `agent-stderr.txt` | What the agent's command wrote, where it wrote anything |

What the game container writes beside its protocol is not kept; where a game fails to start or ends without saying how, it is in the error.

An agent's image may be any Linux image with `sh` and `sleep`, which the idle container and the gate need. `agent/Containerfile` builds `minigames-agent`, one to build on: slim Debian with Python, `pyautogui` on `python-xlib`, Tk, `scrot`, `xdotool`, `x11-utils`, `XDG_SESSION_TYPE=x11` for `pyautogui.screenshot()`, and the [harness](#the-harness-in-the-agent-image). The pinned packages beside `pyautogui` are the ones it imports.

## The Harness In The Agent Image

`agent/harness.py` is installed in the agent image as `harness`. It is a deliberately minimal computer-use harness, a placeholder for your own:

```
harness [--base-url URL] [--model NAME] clock|ticks TASK
```

- Each turn it takes a screenshot with `pyautogui`, sends it with the task to an OpenAI-compatible chat-completions endpoint, and runs the Python that comes back with `pyautogui` and `time` in hand. It waits 20 ms, and takes the next screenshot.
- Every request is the system prompt, the task, and one screenshot. Nothing carries over between turns: not earlier answers, and not what the code did.
- Fenced blocks are run where there are any, and the whole answer where there are none. An answer that does not compile is skipped, and errors are ignored.
- `clock` and `ticks` differ only in the system prompt and in that `ticks` presses F1 after running each answer. The `ticks` prompt says the game reads what is held at each press, so a press, hold, or drag spans answers.
- `pyautogui`'s pause after every call is off, and so is its abort when the pointer reaches a corner.
- The endpoint is `--base-url` or `OPENAI_BASE_URL`, the model `--model` or `OPENAI_MODEL`, and the key `OPENAI_API_KEY`. A request that fails is written to standard error and tried again a second later.

## The Command Line

`python -m minigames.bench`, or `minigames-bench` where installed.

| Flag | What it does | Default |
|---|---|---|
| `--run-agent IMAGE COMMAND` | The agent: a container from `IMAGE`, started by `COMMAND` with the task in place of `{task}` | required |
| `--engine NAME` | `podman` or `docker` | required |
| `--clock N` | Ticks of game time per second of the agent's, from 1 to 60 | 60 |
| `--ticks [N]` | Instead of `--clock`: the game stands still, and each press of F1 lets `N` ticks pass, from 1 to 60, or 1 where `N` is left out | off, so the pace is `--clock` |
| `--idle-timeout SECONDS` | With `--ticks`: stop an episode once this long passes without a press of F1 | none |
| `--total-timeout SECONDS` | With `--ticks`: stop an episode this long after it began | none |
| `--games NAME...` | Which games to play | every game |
| `--tags TAG...` | Instead of `--games`: every game carrying any of these tags, from [TAGS.md](TAGS.md) | |
| `--seed N` | Which layout every game is played on | `0` |
| `--out DIR` | The folder the run is written into | `runs/<start time>` |
| `--record` | Write `recording.mp4` for every game, a frame a tick | off |
| `--jobs N` | Play `N` games at once, each with an agent of its own | 1 |
| `--keep-going` | When a game cannot be played, write `error.txt` in its folder and play on | the run stops |
| `--resume DIR` | Play what a stopped run in `DIR` had not finished | |
| `-v`, `--verbose` | A progress bar while the run plays | off |
| `--list` | Print every game, how it is scored, and its tags, then stop | |
| `--report DIR...` | Print finished runs side by side, then stop | |

A run's games are played by `containers.sweep`. Stopping it kills every container it started and keeps what had finished. At the end the run prints every game's outcome, ticks, and seconds, and how many passed.

```
python -m minigames.bench --run-agent minigames-agent \
    "harness --base-url http://host.containers.internal:PORT/v1 --model MODEL ticks {task}" \
    --engine podman --ticks 3 --idle-timeout 180 --total-timeout 10800
python -m minigames.bench --run-agent my-agent-image "my-agent {task}" --engine docker \
    --clock 20 --tags timing --jobs 4
```

## What A Run Writes

```
runs/20260928-045517/     the start time, or --out
  config.json             written at the start, and again on --resume
  <game>/                 one folder for each game
    result.json
    inputs.txt
    agent-stdout.txt      where the agent wrote anything
    agent-stderr.txt      where the agent wrote anything
    recording.mp4         with --record
    error.txt             with --keep-going, where the game could not be played
  summary.json            written once every game has been played
```

| `config.json` | What it holds |
|---|---|
| `version` | The `minigames` version that ran it |
| `started`, `resumed` | When the run began, and when it was resumed, as UTC times |
| `command` | The command line, less `--out` and its folder, which `--resume` plays again |
| `image`, `agent` | The agent's image and its command, `{task}` and all |
| `engine`, `pace` | `podman` or `docker`, and `clock:N` or `ticks:N` |
| `games` | The games, in the order they are played |
| `idle_timeout`, `total_timeout` | The timeouts in seconds, or `null` |
| `record` | Whether every game is filmed |

| `summary.json` | What it holds |
|---|---|
| `ended` | When the last game ended, as a UTC time |
| `passed`, `failed`, `incomplete` | The games won, the games lost, and the rest: stopped before they ended, or not played for an error |
| `score`, `total` | How many passed, out of how many games |

A run with no `summary.json` has not finished; `--resume` plays the games with no `result.json`, and writes it. `--report` reads `config.json`, `summary.json`, and every `result.json`. A recording is made after the game ends, by `minigames.replay` from `inputs.txt`, and encoded by the ffmpeg in `imageio-ffmpeg`.

## Scoring

- A finish inside the limit is worth one; anything else is worth nothing. Time is reported beside the score.
- A run plays each game once, on one seed, with one agent at one pace.
- The seed is never shown to the agent, though every `result.json` carries it.

## The Benchmark

The benchmark is every game played at four paces, a run at each:

| Pace | The game | Its 180-second limit takes |
|---|---|---|
| `--ticks 3` | Stands still, and each press of F1 lets 3 ticks pass | 3600 presses of F1 |
| `--clock 6` | Runs ten times slower | 30 minutes |
| `--clock 20` | Runs three times slower | 9 minutes |
| `--clock 60` | Runs at full speed | 3 minutes |

```
python -m minigames.bench --run-agent IMAGE "COMMAND {task}" --engine podman --ticks 3 --out runs/ticks3
python -m minigames.bench --run-agent IMAGE "COMMAND {task}" --engine podman --clock 6 --out runs/clock6
python -m minigames.bench --run-agent IMAGE "COMMAND {task}" --engine podman --clock 20 --out runs/clock20
python -m minigames.bench --run-agent IMAGE "COMMAND {task}" --engine podman --clock 60 --out runs/clock60
python -m minigames.bench --report runs/ticks3 runs/clock6 runs/clock20 runs/clock60
```

On a clock, a game runs until it is won or lost or its limit passes, so a game takes up to its limit in the agent's time from the table, and a run up to that for every game. In ticks, a game waits on its agent, so a run lasts as long as the agent takes over every game, unless `--idle-timeout` or `--total-timeout` stops an episode sooner. `--jobs` plays several games at once.

## What Is Pinned

`Cargo.lock` fixes the Rust side and `uv.lock` the Python side, where `imageio-ffmpeg` is the `bench` extra. The agent image pins `pyautogui` and what it runs on.
