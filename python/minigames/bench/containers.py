"""An agent in a container of its own playing a game in another, the two sharing a display and
nothing else."""

from __future__ import annotations

import contextlib
import json
import os
import shlex
import shutil
import subprocess
import tempfile
import threading
import time
import traceback
import uuid
from concurrent.futures import ThreadPoolExecutor, as_completed, wait
from pathlib import Path
from typing import Callable, Iterable

import minigames

from .runner import encode, specs

GAME = "minigames-game"
"""The image a game is played in, built from the Containerfile's `game` stage."""

GATE = "minigames-gate"
"""What the agent's container prints once it is holding the agent's command back until `go`."""

BEGUN = 60.0
"""Seconds a container has to come up before the episode is given up on."""

LINGER = 1.0
"""Seconds a game in ticks is given, after the agent's command ends, to take its last presses."""

PACES = ("clock", "ticks")
"""A clock of ticks per second of the agent's, or ticks per press of F1."""

DETACHED = (
    {"creationflags": subprocess.CREATE_NEW_PROCESS_GROUP}
    if os.name == "nt"
    else {"start_new_session": True}
)
"""How a container's client is started: in a session of its own, out of reach of an interrupt."""

HOST = {"docker": ["--add-host", "host.docker.internal:host-gateway"]}
"""What an engine needs told before an agent's container can reach this machine by name."""


class ContainerError(RuntimeError):
    pass


ENGINES = ("podman", "docker")
"""The engines a run can be played on, one of which is always named."""


def installed(engine: str) -> None:
    """Refuse an engine that is not one of `ENGINES`, or is not installed."""
    if engine not in ENGINES:
        raise ContainerError(f"the engine is one of {', '.join(ENGINES)}, not {engine}")
    if shutil.which(engine) is None:
        raise ContainerError(f"{engine} is not installed")


def check(engine: str, *images: str) -> None:
    """Refuse to begin where an image has not been built."""
    for image in images:
        found = subprocess.run([engine, "image", "inspect", image], capture_output=True)
        if found.returncode != 0:
            raise ContainerError(f"there is no image called {image}; build it first (README.md)")


def paced(pace: tuple[str, int]) -> list[str]:
    """`pace` as the game image's command line takes it, refused where it is not one."""
    kind, count = pace
    if kind not in PACES:
        raise ValueError(f"a pace is a clock or ticks, not {kind}")
    if not 1 <= count <= minigames.tick_hz():
        raise ValueError(f"--{kind} is from 1 to {minigames.tick_hz()}, not {count}")
    return [f"--{kind}", str(count)]


def timeouts(pace: tuple[str, int], idle: float | None, total: float | None) -> None:
    """Refuse a timeout on a clock, or one that is not a positive number of seconds."""
    given = [seconds for seconds in (idle, total) if seconds is not None]
    if given and pace[0] != "ticks":
        raise ValueError("a game on a clock ends by itself, and takes no timeout")
    if any(not seconds > 0 for seconds in given):
        raise ValueError(f"a timeout is a positive number of seconds, not {min(given)}")


def task(engine: str, game: str, pace: tuple[str, int], label: str = "minigames.run") -> str:
    """What the agent is told, as the game image tells it."""
    told = subprocess.run(
        [engine, "run", "--rm", "--pull", "never", "--label", label, "--network", "none", GAME,
         "task", game, *paced(pace)],
        capture_output=True,
        text=True,
        **DETACHED,
    )
    if told.returncode != 0:
        raise ContainerError(f"the game image could not say what {game} asks: {told.stderr}")
    return told.stdout.rstrip("\n")


def command(template: str, task: str) -> list[str]:
    """The agent's command as words, with the task put whole in place of `{task}`."""
    words = shlex.split(template)
    if not any("{task}" in word for word in words):
        raise ValueError(f"the agent's command has no {{task}} to put the task in: {template}")
    return [word.replace("{task}", task) for word in words]


def play(
    engine: str,
    image: str,
    template: str,
    game: str,
    seed: int = 0,
    pace: tuple[str, int] = ("clock", minigames.tick_hz()),
    folder: Path | None = None,
    record: bool = False,
    run_label: str = "minigames.run",
    stop: threading.Event | None = None,
    idle_timeout: float | None = None,
    total_timeout: float | None = None,
) -> dict:
    """One episode on `engine` at `pace`. Both containers are brought up and left idle, then the
    game and the agent's command are let go at the same moment. On a clock the game runs to its
    own end whatever the agent does. In ticks it also ends when the agent's command does, when
    `idle_timeout` seconds pass without a press of F1, or `total_timeout` seconds after it
    began, and the result says which. Then the agent is stopped. Every action that landed on
    the game, the result, and what the agent's command wrote, where it wrote anything, go into
    `folder`, and the episode is filmed where `record` asks. Every container and volume also carries `run_label`, and `stop`, once set,
    keeps the episode from starting anything more."""
    arguments = paced(pace)
    timeouts(pace, idle_timeout, total_timeout)
    spec = specs().get(game)
    if spec is None:
        raise ValueError(f"no game called {game}")
    if folder is None:
        with tempfile.TemporaryDirectory() as scratch:
            return play(
                engine, image, template, game, seed, pace, Path(scratch), record, run_label,
                stop, idle_timeout, total_timeout,
            )
    installed(engine)
    check(engine, GAME, image)
    words = command(template, task(engine, game, pace, run_label))
    folder.mkdir(parents=True, exist_ok=True)

    def going() -> None:
        if stop is not None and stop.is_set():
            raise ContainerError("the run was stopped")

    token = uuid.uuid4().hex[:12]
    label = f"minigames.episode={token}"
    volume = f"minigames-{token}"
    agent, played = f"{volume}-agent", f"{volume}-game"
    shared = ["--label", label, "--label", run_label, "--security-opt", "label=disable",
              "--security-opt", "no-new-privileges",
              "-v", f"{volume}:/tmp/.X11-unix"]
    width, height = minigames.size()
    stdout, stderr = folder / "agent-stdout.txt", folder / "agent-stderr.txt"
    game_errors = tempfile.TemporaryFile("w+")

    def complaint() -> str:
        game_errors.seek(0)
        return game_errors.read()

    screen = gate = total = None
    stopped: list[str] = []
    over, pressed = threading.Event(), threading.Event()
    stopping = threading.Lock()
    try:
        going()
        run(engine, "volume", "create", "--label", label, "--label", run_label, volume)
        going()
        run(engine, "run", "-d", "--pull", "never", "--name", agent, *shared, *HOST.get(engine, []),
            "-e", "DISPLAY=:0", "-e", f"WIDTH={width}", "-e", f"HEIGHT={height}",
            "--entrypoint", "sleep", image, "infinity")
        going()
        with open(stdout, "w") as out, open(stderr, "w") as err:
            screen = subprocess.Popen(
                [engine, "run", "-i", "--rm", "--pull", "never", "--name", played, *shared,
                 "--network", "none", GAME, "screen", game, "--seed", str(seed), *arguments],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=game_errors, text=True,
                **DETACHED,
            )
            late = threading.Timer(BEGUN, lambda: (quiet(engine, "kill", played), screen.kill()))
            late.start()
            ready = screen.stdout.readline().strip()
            late.cancel()
            if ready != "ready":
                raise ContainerError(f"the game did not start: {complaint()}")
            gate = subprocess.Popen(
                [engine, "exec", "-i", agent, "sh", "-c", f'echo {GATE}; read _; exec "$@"', "sh",
                 *words],
                stdin=subprocess.PIPE, stdout=out, stderr=err, **DETACHED,
            )
            held_back(gate, stdout)
            going()
            screen.stdin.write("go\n")
            screen.stdin.flush()
            gate.stdin.write(b"\n")
            gate.stdin.close()
            began = time.perf_counter()

            def halt(why: str) -> None:
                with stopping, contextlib.suppress(OSError, ValueError):
                    if not over.is_set() and not stopped:
                        stopped.append(why)
                        screen.stdin.write("stop\n")
                        screen.stdin.flush()

            def linger() -> None:
                time.sleep(LINGER)
                halt("agent-exited")

            def idle() -> None:
                while not over.is_set():
                    if not pressed.wait(idle_timeout):
                        halt("idle-timeout")
                        return
                    pressed.clear()

            if total_timeout is not None:
                total = threading.Timer(total_timeout, halt, ["total-timeout"])
                total.daemon = True
                total.start()
            if idle_timeout is not None:
                threading.Thread(target=idle, daemon=True).start()
            if pace[0] == "ticks":
                threading.Thread(target=lambda: (gate.wait(), linger()), daemon=True).start()
            lines = []
            for line in screen.stdout:
                if line == "advanced\n":
                    pressed.set()
                else:
                    lines.append(line.rstrip("\n"))
            with stopping:
                over.set()
            screen.wait()
            wall = time.perf_counter() - began
            said = complaint()
    finally:
        game_errors.close()
        over.set()
        pressed.set()
        if total is not None:
            total.cancel()
        for name in (agent, played):
            quiet(engine, "kill", name)
        for client in (screen, gate):
            if client is not None:
                settle(client)
        for name in (agent, played):
            quiet(engine, "rm", "-f", name)
        quiet(engine, "volume", "rm", "-f", volume)
    stdout.write_bytes(stdout.read_bytes().removeprefix(f"{GATE}\n".encode()))
    for log in (stdout, stderr):
        if log.stat().st_size == 0:
            log.unlink()

    if not lines or not lines[-1].startswith("over "):
        raise ContainerError(f"the game ended without saying how: {said}")
    _, outcome, ticks = lines[-1].split()
    ticks = int(ticks)
    last = "unfinished" if outcome == "playing" else outcome
    (folder / "inputs.txt").write_text("\n".join([*lines[:-1], f"over {last} {ticks}"]) + "\n")
    inputs = [(int(tick), line) for tick, line in (entry.split(" ", 1) for entry in lines[:-1])]
    if record:
        again = encode(folder, lambda write: minigames.replay(game, seed, inputs, ticks, write))
    else:
        again = minigames.replay(game, seed, inputs, ticks)
    if again != (outcome, ticks):
        raise ContainerError(
            f"played again here, {game} ended {again} rather than {(outcome, ticks)}: the game "
            "image was built from other code than this; build it again"
        )
    if outcome == "playing":
        outcome = "unfinished"
    result = {
        "game": game,
        "seed": seed,
        "outcome": outcome,
        "ticks": ticks,
        "seconds": round(wall, 3),
        "stopped": stopped[0] if stopped and outcome == "unfinished" else None,
    }
    (folder / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


def sweep(
    engine: str,
    image: str,
    template: str,
    games: Iterable[str],
    seed: int = 0,
    pace: tuple[str, int] = ("clock", minigames.tick_hz()),
    out: Path | None = None,
    record: bool = False,
    jobs: int = 1,
    each: Callable[[dict[str, dict]], None] | None = None,
    failed: dict[str, str] | None = None,
    idle_timeout: float | None = None,
    total_timeout: float | None = None,
) -> dict[str, dict]:
    """Every game in `games` played once on `seed`, `jobs` of them at a time, each in a folder
    of its own under `out`, with `each` handed the results so far as they come in. Where
    `failed` is given, a game that raises is added to it with what went wrong, written beside
    it, and the run goes on; otherwise the first one to raise stops the run. However the run
    stops, its containers are killed, and what had finished by then is kept."""
    if jobs < 1:
        raise ValueError(f"a run takes at least one job, not {jobs}")
    games = list(games)
    timeouts(pace, idle_timeout, total_timeout)
    installed(engine)
    check(engine, GAME, image)
    label = f"minigames.run={uuid.uuid4().hex[:12]}"
    stop = threading.Event()
    results: dict[str, dict] = {}

    def one(game: str) -> tuple[str, dict | None, str | None]:
        where = out / game if out else None
        if where is not None and where.exists():
            shutil.rmtree(where)
        try:
            played = play(
                engine, image, template, game, seed, pace, where, record, label, stop,
                idle_timeout, total_timeout,
            )
            return game, played, None
        except Exception as error:
            if failed is None or stop.is_set():
                raise
            if where is not None:
                where.mkdir(parents=True, exist_ok=True)
                (where / "error.txt").write_text(traceback.format_exc())
            return game, None, f"{type(error).__name__}: {error}"

    def landed(outcome: tuple[str, dict | None, str | None]) -> None:
        game, played, failure = outcome
        if played is not None:
            results[game] = played
            if each is not None:
                each(results)
        if failure is not None:
            failed[game] = failure

    with ThreadPoolExecutor(jobs) as pool:
        pending = [pool.submit(one, game) for game in games]
        taken = set()
        try:
            for done in as_completed(pending):
                taken.add(done)
                landed(done.result())
        except BaseException:
            stop.set()
            for later in pending:
                later.cancel()
            while not all(later.done() for later in pending):
                listed = [engine, "ps", "-a", "-q", "--filter", f"label={label}"]
                for left in subprocess.run(listed, capture_output=True, text=True).stdout.split():
                    quiet(engine, "kill", left)
                wait(pending, timeout=1)
            for later in pending:
                if later not in taken and not later.cancelled() and not later.exception():
                    landed(later.result())
            raise
    return {game: results[game] for game in games if game in results}


def run(engine: str, *words: str) -> None:
    done = subprocess.run([engine, *words], capture_output=True, text=True, **DETACHED)
    if done.returncode != 0:
        raise ContainerError(f"{engine} {' '.join(words[:2])} failed: {done.stderr.strip()}")


def quiet(engine: str, *words: str) -> None:
    subprocess.run([engine, *words], capture_output=True, **DETACHED)


def settle(client: subprocess.Popen) -> None:
    """A container's client waited for once its container is gone, and its pipes closed."""
    try:
        client.wait(timeout=30)
    except subprocess.TimeoutExpired:
        client.kill()
        client.wait()
    for pipe in (client.stdin, client.stdout):
        if pipe is not None:
            with contextlib.suppress(OSError):
                pipe.close()


def held_back(gate: subprocess.Popen, said: Path) -> None:
    """Waits until the agent's container is holding its command at the gate."""
    deadline = time.monotonic() + BEGUN
    while not said.read_text().startswith(GATE):
        if gate.poll() is not None:
            raise ContainerError("the agent's container could not run its command through sh")
        if time.monotonic() > deadline:
            raise ContainerError("the agent's container never came up")
        time.sleep(0.005)
