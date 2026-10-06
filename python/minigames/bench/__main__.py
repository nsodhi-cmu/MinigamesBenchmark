"""Run an agent in a container over the catalog and print what it managed."""

from __future__ import annotations

import argparse
import datetime
import importlib.metadata
import json
import os
import sys
import time
from pathlib import Path

import minigames

from . import containers
from .runner import chosen, tagged
from .scoring import summarize


class Progress:
    """A bar on standard error, and a line for each game as it finishes."""

    WIDTH = 30

    def __init__(self, total: int, done: int = 0) -> None:
        self.total = total
        self.started = time.monotonic()
        self.live = sys.stderr.isatty()
        self.show("", done, 0)

    def __call__(self, results: dict[str, dict]) -> None:
        game, result = list(results.items())[-1]
        solved = sum(result["outcome"] == "won" for result in results.values())
        self.show(f"{game}: {ended(result)}", len(results), solved)

    def show(self, said: str, done: int, solved: int) -> None:
        filled = self.WIDTH * done // max(self.total, 1)
        elapsed = int(time.monotonic() - self.started)
        bar = (
            f"[{'#' * filled}{'.' * (self.WIDTH - filled)}] {done}/{self.total} games, "
            f"{solved} passed, {elapsed // 60}m{elapsed % 60:02d}s"
        )
        if self.live:
            sys.stderr.write(f"\r\033[K{said}\n{bar}" if said else bar)
        elif said:
            sys.stderr.write(f"{said}  {bar}\n")
        sys.stderr.flush()

    def close(self) -> None:
        if self.live:
            sys.stderr.write("\n")


def ended(result: dict) -> str:
    """How a game ended, with what stopped it where something did."""
    stopped = f" ({result['stopped']})" if result["stopped"] else ""
    return f"{result['outcome']}{stopped}"


def table(results: dict[str, dict]) -> None:
    """Every game played, on a line of its own."""
    print(f"{'GAME':<10} {'OUTCOME':<26} {'TICKS':>6} {'SECONDS':>9}")
    for game, result in results.items():
        print(f"{game:<10} {ended(result):<26} {result['ticks']:>6} {result['seconds']:>9.1f}")


def write(path: Path, data: dict) -> None:
    """`data` as JSON at `path`, whole or not at all."""
    partway = path.with_name(path.name + ".partway")
    partway.write_text(json.dumps(data, indent=2) + "\n")
    os.replace(partway, path)


def finished(out: Path, games: list[str]) -> dict[str, dict]:
    """The result of every game in `games` that `out` holds one for, in that order."""
    return {
        game: json.loads((out / game / "result.json").read_text())
        for game in games
        if (out / game / "result.json").exists()
    }


def compare(folders: list[Path]) -> None:
    """Finished runs side by side: how each game went in each, then how each run went."""
    runs = []
    for where in folders:
        try:
            config = json.loads((where / "config.json").read_text())
            summary = json.loads((where / "summary.json").read_text())
        except (OSError, ValueError) as missing:
            raise SystemExit(f"no finished run in {where}: {missing}") from None
        runs.append((config, summary, finished(where, config["games"])))
    for number, (where, (config, _, _)) in enumerate(zip(folders, runs), start=1):
        print(f"run {number}: {config['image']}: {config['agent']} at {config['pace']} ({where})")
    wide = 18
    numbers = [f"run {number}" for number in range(1, len(runs) + 1)]
    print(f"\n{'':<12}" + "".join(f"{number:>{wide}}" for number in numbers))

    def line(name: str, cells: list[str]) -> None:
        print(f"{name:<12}" + "".join(f"{cell:>{wide}}" for cell in cells))

    def cell(game: str, config: dict, results: dict[str, dict]) -> str:
        if game not in config["games"]:
            return "-"
        result = results.get(game)
        if result is None:
            return "error"
        if result["outcome"] == "won":
            return f"won in {result['ticks'] / minigames.tick_hz():.1f}s"
        return result["outcome"]

    for game in sorted({game for config, _, _ in runs for game in config["games"]}):
        line(game, [cell(game, config, results) for config, _, results in runs])
    print()
    line("passed", [f"{summary['score']}/{summary['total']}" for _, summary, _ in runs])
    line("seconds", [
        f"{sum(result['seconds'] for result in results.values()):.0f}s" for _, _, results in runs
    ])


def without_out(words: list[str]) -> list[str]:
    """A command line with `--out` and its folder taken out, in any form argparse takes."""
    kept, skipping = [], False
    for word in words:
        if skipping:
            skipping = False
            continue
        flag = word.split("=", 1)[0]
        if len(flag) > 2 and "--out".startswith(flag):
            skipping = "=" not in word
            continue
        kept.append(word)
    return kept


RUN = (
    "run_agent", "engine", "games", "tags", "seed", "clock", "ticks", "idle_timeout",
    "total_timeout", "out", "record",
)


def main() -> int:
    parser = argparse.ArgumentParser(prog="minigames-bench", description=__doc__)
    parser.add_argument(
        "--run-agent",
        nargs=2,
        metavar=("IMAGE", "COMMAND"),
        help="the agent: a container from IMAGE, started by COMMAND with the task in place of "
        "{task}",
    )
    parser.add_argument(
        "--engine",
        choices=containers.ENGINES,
        metavar="NAME",
        help="which of the two plays the containers, podman or docker",
    )
    pacing = parser.add_mutually_exclusive_group()
    pacing.add_argument(
        "--clock",
        type=int,
        metavar="N",
        help="ticks of game time per second of the agent's, from 1 to 60; 60 if neither this "
        "nor --ticks is given",
    )
    pacing.add_argument(
        "--ticks",
        type=int,
        nargs="?",
        metavar="N",
        const=1,
        help="instead: the game stands still, and each press of F1 lets this many ticks pass, "
        "from 1 to 60; 1 if given without a number",
    )
    parser.add_argument(
        "--idle-timeout",
        type=float,
        metavar="SECONDS",
        help="with --ticks: stop an episode once this long passes without a press of F1",
    )
    parser.add_argument(
        "--total-timeout",
        type=float,
        metavar="SECONDS",
        help="with --ticks: stop an episode this long after it began",
    )
    choosing = parser.add_mutually_exclusive_group()
    choosing.add_argument(
        "--games", nargs="+", metavar="NAME", help="which games, all of them if left out"
    )
    choosing.add_argument(
        "--tags",
        nargs="+",
        metavar="TAG",
        help="instead: every game that carries any of these tags",
    )
    parser.add_argument(
        "--seed", type=int, metavar="N", help="which layout every game is played on, 0 if left out"
    )
    parser.add_argument(
        "--out",
        type=Path,
        metavar="DIR",
        help="the folder the run is written into, runs/<when it started> if left out",
    )
    parser.add_argument(
        "--record", action="store_true", help="write every episode as an mp4, a frame a tick"
    )
    parser.add_argument(
        "--jobs",
        type=int,
        metavar="N",
        default=1,
        help="how many games to play at once, each with its own agent",
    )
    parser.add_argument(
        "--keep-going",
        action="store_true",
        help="when a game cannot be played, write why beside it and play on instead of stopping",
    )
    parser.add_argument(
        "--resume",
        type=Path,
        metavar="DIR",
        help="play whatever a stopped run in DIR had not finished, as it was begun",
    )
    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="show a progress bar while the run plays",
    )
    parser.add_argument("--list", action="store_true", help="print the catalog and stop")
    parser.add_argument(
        "--report",
        type=Path,
        nargs="+",
        metavar="DIR",
        help="print finished runs side by side and stop",
    )
    arguments = parser.parse_args()

    if arguments.list:
        every = minigames.games()
        wide = max(len(spec["title"]) for spec in every)
        for spec in every:
            print(
                f"{spec['name']:<10} {spec['title']:<{wide}} {spec['scoring']:<9} "
                f"{', '.join(spec['tags'])}"
            )
        return 0
    if arguments.report:
        compare(arguments.report)
        return 0

    now = datetime.datetime.now(datetime.timezone.utc)
    config = None
    if arguments.resume:
        given = [
            name for name in RUN if getattr(arguments, name) is not None
            and getattr(arguments, name) is not False
        ]
        if given:
            flags = ", ".join("--" + name.replace("_", "-") for name in given)
            parser.error(f"--resume plays a run as it was begun, and takes no {flags}")
        try:
            config = json.loads((arguments.resume / "config.json").read_text())
        except (OSError, ValueError) as missing:
            raise SystemExit(f"no run to resume in {arguments.resume}: {missing}") from None
        begun = parser.parse_args(config["command"])
        for name in RUN:
            setattr(arguments, name, getattr(begun, name))
        arguments.out = arguments.resume
        config["resumed"].append(now.isoformat(timespec="seconds"))

    if arguments.run_agent is None:
        parser.error("name the agent with --run-agent IMAGE COMMAND")
    if arguments.engine is None:
        parser.error(f"--run-agent wants --engine, one of {', '.join(containers.ENGINES)}")
    if arguments.jobs < 1:
        parser.error("a run takes at least one job")
    clock = minigames.tick_hz() if arguments.clock is None else arguments.clock
    pace = ("clock", clock) if arguments.ticks is None else ("ticks", arguments.ticks)
    seed = 0 if arguments.seed is None else arguments.seed
    image, template = arguments.run_agent
    out = arguments.out or Path("runs") / now.strftime("%Y%m%d-%H%M%S")

    try:
        containers.paced(pace)
        containers.timeouts(pace, arguments.idle_timeout, arguments.total_timeout)
        containers.command(template, "")
        containers.installed(arguments.engine)
        containers.check(arguments.engine, containers.GAME, image)
        if config is None:
            config = {
                "version": importlib.metadata.version("minigames"),
                "started": now.isoformat(timespec="seconds"),
                "resumed": [],
                "command": without_out(sys.argv[1:]),
                "image": image,
                "agent": template,
                "engine": arguments.engine,
                "pace": ":".join(map(str, pace)),
                "games": chosen(tagged(arguments.tags) if arguments.tags else arguments.games),
                "idle_timeout": arguments.idle_timeout,
                "total_timeout": arguments.total_timeout,
                "record": arguments.record,
            }
    except (ValueError, containers.ContainerError) as refusal:
        raise SystemExit(str(refusal)) from None

    planned = config["games"]
    out.mkdir(parents=True, exist_ok=True)
    write(out / "config.json", config)
    (out / "summary.json").unlink(missing_ok=True)
    results = finished(out, planned)
    progress = Progress(len(planned), len(results)) if arguments.verbose else None
    failed: dict[str, str] = {}

    def arrived(played: dict[str, dict]) -> None:
        if progress is not None:
            progress({**results, **played})

    try:
        played = containers.sweep(
            arguments.engine,
            image,
            template,
            [game for game in planned if game not in results],
            seed=seed,
            pace=pace,
            out=out,
            record=arguments.record,
            jobs=arguments.jobs,
            each=arrived,
            failed=failed if arguments.keep_going else None,
            idle_timeout=arguments.idle_timeout,
            total_timeout=arguments.total_timeout,
        )
    except (ValueError, containers.ContainerError) as refusal:
        raise SystemExit(f"{refusal}\nwhat finished is in {out}; --resume {out} plays the rest")
    except BaseException:
        print(f"what finished is in {out}; --resume {out} plays the rest", file=sys.stderr)
        raise
    finally:
        if progress is not None:
            progress.close()
    results.update(played)
    results = {game: results[game] for game in planned if game in results}
    ended_at = datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")
    summary = summarize(planned, results, ended_at)
    write(out / "summary.json", summary)

    table(results)
    print(
        f"\n{image}: {template} at {config['pace']}: "
        f"{summary['score']}/{summary['total']} passed"
    )
    for game, error in failed.items():
        print(f"error: {game}: {error}")
    print(f"written to {out}")
    if failed:
        print(f"--resume {out} plays the games with an error again")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
