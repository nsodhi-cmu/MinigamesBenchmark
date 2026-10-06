---
name: minigames-setup
description: Use when a user wants to benchmark an agent or model on minigames, or asks how to set it up, which flags or paces to choose, how long a run takes, or how to read a run's results. Covers installing what is needed, building the container images, wiring an agent and a model endpoint, choosing parameters, warning about costs, and interpreting results. Setup and advice only.
---

# Setting up the minigames benchmark

minigames is a benchmark of small games played from pixels alone. An agent sees a 640×480
screen in a container and works its mouse and keyboard; a second container plays the game.
Each game is passed or not: won inside its limit of 180 seconds of game time.

## Your role

You prepare the user's machine and hand them exact commands. You never start a benchmark run,
a trial or dry run, a single game, or a request to a model. Runs take hours and can spend
money; starting one is the user's decision. If the user explicitly asks you to run something,
that is their call: first show the command, its worst-case duration, and the costs below.

## What is needed

- **Linux.** Only Linux is tested. macOS and Windows may work but are not supported.
- **podman or docker.** Either one; images built with one are invisible to the other.
- **Rust** (rustup, stable). Builds the Python extension. A build that fails with
  "linker `cc` not found" needs the system C compiler (`build-essential`, `gcc`, or similar).
- **uv**, for the Python environment.

Install what is missing only with the user's agreement, and tell them what you installed.

## Building

From the repository root:

- `uv sync` builds the Python extension and the `minigames-bench` command. Use
  `uv sync --extra bench` only if the user wants video recordings.
- The game image: `podman build -t minigames-game .`
  With docker: `docker build -f Containerfile -t minigames-game .`
- The agent image: `podman build -t minigames-agent agent`
  With docker: `docker build -f agent/Containerfile -t minigames-agent agent`

Builds take a few minutes and are safe to repeat. Rebuild both images whenever the repository
changes: a run refuses to score a game whose image was built from different code.

## The agent

`minigames-agent` is Debian with Python, `pyautogui` on `python-xlib`, `xdotool`, `x11-utils`,
`scrot`, Tk, and `harness`, a deliberately minimal agent.

**Using `harness`.** It takes a screenshot, sends it with the task to an OpenAI-compatible
chat-completions endpoint, runs the Python that comes back, and remembers nothing between
turns. Its command is `harness --base-url URL --model NAME clock|ticks {task}`. Its mode must
match the run's pace: `clock` for `--clock`, `ticks` for `--ticks`. The endpoint, model, and
key can also come from `OPENAI_BASE_URL`, `OPENAI_MODEL`, and `OPENAI_API_KEY`.

**Using the user's own agent.** Write a Containerfile `FROM minigames-agent`, or from any
Linux image with `sh` and `sleep`; install the agent; build it under a name of the user's
choosing. Inside the container:

- the display is `DISPLAY=:0`, 640×480, with `WIDTH=640` and `HEIGHT=480` set;
- input goes through X11: `xdotool`, `pyautogui`, or XTEST;
- the agent's command gets the full task text, as one argument, in place of `{task}`. The task
  says the screen size, the pace, and how to play the game;
- the command is split into words once, never by a shell. For pipes or variables use
  `sh -c '... "$0"' {task}`;
- the command must contain `{task}`.

**Reaching a model.** The agent's container has the engine's default network; the game's
container has none. A model server on the user's machine is `host.containers.internal` under
podman and `host.docker.internal` under docker, and must listen on `0.0.0.0`, not `127.0.0.1`.

**Secrets.** The agent's command is saved in the run's `config.json`, which people share. Put
API keys in the image as `ENV` lines, never in the command, and warn that such an image must
not be pushed anywhere public.

## Paces

The pace decides how game time relates to the agent's time.

- `--clock N`, from 1 to 60: the game runs N ticks per second of real time, whatever the agent
  does. 60 is full speed. Slower clocks give a slow agent more real time per game second.
- `--ticks N`, from 1 to 60 (bare `--ticks` is 1): the game stands still until the agent
  presses F1, then lets N ticks pass. The agent must hold a key or button down across a press
  of F1; one pressed and released between two presses does nothing but move the pointer.

The official benchmark is four runs over every game, on seed 0:
`--ticks 3`, `--clock 6`, `--clock 20`, and `--clock 60`. Only results at these settings are
comparable with published ones.

## Every flag

| Flag | Meaning | Advice |
|---|---|---|
| `--run-agent IMAGE COMMAND` | The agent's image and its command, with `{task}` | Required |
| `--engine podman\|docker` | Which engine plays the containers | Required; the one the images were built with |
| `--clock N` / `--ticks N` | The pace; `--clock 60` if neither is given | Official settings above |
| `--idle-timeout SECONDS` | Ticks only: stop a game once this long passes without a press of F1 | Set it, so a stuck agent cannot stall the run |
| `--total-timeout SECONDS` | Ticks only: stop a game this long after it began | Set it to cap time and spend per game |
| `--games NAME...` / `--tags TAG...` | Which games; all of them by default | `--list` prints every game with its tags |
| `--seed N` | Which layout every game uses; 0 by default | Keep 0 for comparable results |
| `--out DIR` | Where the run is written; `runs/<UTC start time>` by default | |
| `--record` | Film every game as `recording.mp4` | Needs `uv sync --extra bench`; costs disk |
| `--jobs N` | Play N games at once, each with its own agent | Agents sharing one endpoint slow each other, which on a clock lowers scores, and spend rises N times as fast |
| `--keep-going` | A game that cannot be played is written down and skipped | Good for long runs |
| `--resume DIR` | Play the games a stopped run has no result for | Takes no run flags; it reuses the run's own |
| `--report DIR...` | Print finished runs side by side | |
| `-v` | A progress bar | |

The run command is `uv run minigames-bench ...` (or `uv run python -m minigames.bench ...`).

## Costs and downsides to raise

- **Time.** A game lasts up to 180 seconds of game time. On `--clock N` that is up to
  180 × 60 / N seconds of real time: 3 minutes at 60, 9 at 20, 30 at 6. Over all 70 games the
  worst case is about 3.5, 10.5, and 35 hours. Games that are lost early end early, so real
  runs are usually shorter, but plan for the worst case.
- **Ticks have no time limit.** A game in ticks waits for the agent; at `--ticks 3` a game can
  take up to 3600 presses of F1. Without `--total-timeout`, a slow or stuck agent can run
  for days.
- **Model spend.** `harness` sends one request with one screenshot per turn, for as long as a
  game lasts. Multiply by games, paces, and `--jobs`.
- **Secrets in the command** are saved with the results.
- **Inside its container the agent is unrestricted.** Nothing stops it from opening windows over
  the game, closing it, or installing software, but a score won any way other than by playing
  the game as intended does not count. Make sure the user's agent plays fairly.
- **Only Linux is tested.**

## Reading a run

A run's folder holds:

- `config.json`, written at the start: `version`, `started`, `resumed` (times of each resume),
  `command` (the command line, without `--out`), `image`, `agent`, `engine`, `pace`
  (`clock:N` or `ticks:N`), `games` (in play order), `idle_timeout`, `total_timeout`,
  and `record`.
- one folder per game, holding:
  - `result.json`: `game`, `seed`, `outcome` (`won`, `lost`, or `unfinished`), `ticks` of game
    time (divide by 60 for seconds), `seconds` of real time, and `stopped`, why an unfinished
    game was stopped: `agent-exited`, `idle-timeout`, or `total-timeout`, else `null`;
  - `inputs.txt`: every action that landed, one per line as its tick and the action, then
    `over`, the outcome, and the last tick;
  - `agent-stdout.txt` and `agent-stderr.txt`, only if the agent printed anything;
  - `recording.mp4` with `--record`;
  - `error.txt`, a traceback, if the game could not be played.
- `summary.json`, written once every game has been played: `ended`, `passed`, `failed`
  (lost), `incomplete` (unfinished or not played), `score` (how many passed), and `total`.

No `summary.json` means the run did not finish; `--resume` completes it. A game that passed
did so inside its limit; its `ticks` say how fast. Time is reported, never part of the score.

## Handing over

Once everything is ready, tell the user:

1. what is installed and built, and anything still missing;
2. the exact command or commands, filled in with their image, agent command, engine, pace,
   and any timeouts;
3. the worst-case duration of each, and the spend it implies;
4. where the results will be written, and how to resume or compare runs.

Then stop. The user starts the run.
