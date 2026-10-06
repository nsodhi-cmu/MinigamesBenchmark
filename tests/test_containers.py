"""What an agent in a container of its own is given and what it can do: the task whole, a game
that does not wait for it, and every input it sends, landing in order and playing back."""

import shlex
import subprocess

import pytest

import minigames
from minigames.bench import containers

pytestmark = pytest.mark.containers


def landed(folder):
    lines = (folder / "inputs.txt").read_text().splitlines()[:-1]
    return [(int(tick), line) for tick, line in (entry.split(" ", 1) for entry in lines)]


def leftovers(runs):
    listed = [
        [runs, "ps", "-a", "-q", "--filter", "label=minigames.episode"],
        [runs, "volume", "ls", "-q", "--filter", "label=minigames.episode"],
    ]
    return [name for words in listed for name in subprocess.run(
        words, capture_output=True, text=True, check=True).stdout.split()]


def test_a_game_left_alone_by_its_agent_runs_to_its_own_end(images, tmp_path):
    record = containers.play(images, "minigames-agent", "true {task}", "circles", folder=tmp_path)
    assert (record["outcome"], record["ticks"]) == ("lost", 96), "an untouched ring closes at 96"
    assert landed(tmp_path) == []
    assert not (tmp_path / "agent-stdout.txt").exists(), "an empty log was kept"
    assert leftovers(images) == []


def test_every_input_an_agent_sends_lands_in_order_and_plays_back(images, tmp_path):
    record = containers.play(
        images, "minigames-test-echo", "echo-agent {task}", "circles", pace=("clock", 20),
        folder=tmp_path,
    )
    told = containers.task(images, "circles", ("clock", 20))
    assert (tmp_path / "agent-stdout.txt").read_text() == told, "the task did not arrive whole"
    inputs = landed(tmp_path)
    assert [line for _, line in inputs] == [
        "move 100 100",
        "press 100 100",
        "release 100 100",
        "down LEFT",
        "up LEFT",
        "down SHIFT",
        "down B",
        "up B",
        "up SHIFT",
        "move 200 200",
        "press right 200 200",
        "release right 200 200",
    ]
    held = [tick for tick, line in inputs if line.endswith(" LEFT")]
    assert held[0] < held[1], "a key held for a while landed as a tap"
    again = minigames.replay("circles", 0, inputs, record["ticks"])
    assert again == (record["outcome"], record["ticks"])


def test_an_agent_reading_the_pixels_wins_by_clicking_and_by_holding_keys(images, tmp_path):
    for game, clock in (("circles", 60), ("maze", 20)):
        record = containers.play(
            images,
            "minigames-test-scripted",
            "scripted-agent {task}",
            game,
            pace=("clock", clock),
            folder=tmp_path / game,
        )
        assert record["outcome"] == "won", f"{game}: {record}"


def test_an_agent_that_leaves_a_deaf_child_playing_is_cleared_away_with_the_game(images, tmp_path):
    containers.play(images, "minigames-test-sloppy", "sloppy {task}", "circles", folder=tmp_path)
    assert landed(tmp_path), "the child left behind never played"
    assert leftovers(images) == []


def test_a_container_run_from_the_command_line_films_and_resumes(images, tmp_path, monkeypatch):
    import json

    import imageio_ffmpeg

    from minigames.bench.__main__ import main

    out = tmp_path / "run"
    begun = ["--run-agent", "minigames-agent", "true {task}", "--engine", images, "--record"]
    begun += ["--games", "circles", "--clock", "30", "--out", str(out)]
    monkeypatch.setattr("sys.argv", ["bench", *begun])
    assert main() == 0
    config = json.loads((out / "config.json").read_text())
    assert (config["pace"], config["games"], config["resumed"]) == ("clock:30", ["circles"], [])
    assert str(out) not in config["command"], "the run's own folder was written into it"
    result = json.loads((out / "circles" / "result.json").read_text())
    assert (result["game"], result["seed"], result["outcome"]) == ("circles", 0, "lost")
    assert result["ticks"] == 96
    summary = json.loads((out / "summary.json").read_text())
    assert (summary["failed"], summary["score"], summary["total"]) == (["circles"], 0, 1)
    frames, _ = imageio_ffmpeg.count_frames_and_secs(str(out / "circles" / "recording.mp4"))
    assert frames == result["ticks"] + 1, "the recording is not a frame a tick"

    (out / "circles" / "result.json").unlink()
    monkeypatch.setattr("sys.argv", ["bench", "--resume", str(out)])
    assert main() == 0
    again = json.loads((out / "circles" / "result.json").read_text())
    assert (again["outcome"], again["ticks"]) == ("lost", 96)
    assert len(json.loads((out / "config.json").read_text())["resumed"]) == 1


def test_the_harness_runs_what_the_model_answers_and_remembers_nothing(images, tmp_path):
    def played(pace, game):
        command = (
            f"sh -c 'fake-model {pace} & exec harness --base-url http://127.0.0.1:8000/v1 "
            f"--model fake {pace} \"$1\"' sh {{task}}"
        )
        count = 3 if pace == "ticks" else 60
        containers.play(
            images, "minigames-test-model", command, game, pace=(pace, count),
            folder=tmp_path / pace,
        )
        said = (tmp_path / pace / "agent-stdout.txt").read_text().splitlines()
        assert said and set(said) == {"asked with 2 messages and 1 pictures"}, said
        return landed(tmp_path / pace)

    clocked = [line for _, line in played("clock", "circles")]
    for line in ("move 320 240", "press 320 240", "release 320 240", "down SPACE", "up SPACE"):
        assert line in clocked, clocked
    assert played("ticks", "circles")[:4] == [
        (0, "move 320 240"),
        (0, "press 320 240"),
        (6, "move 100 100"),
        (6, "release 100 100"),
    ], "a held button did not span presses of F1, or a click inside one answer landed"


def test_in_ticks_only_what_is_held_at_each_advance_lands_and_the_agent_leaving_ends_it(
    images, tmp_path
):
    record = containers.play(
        images, "minigames-test-stepper", "stepper-agent {task}", "maze", pace=("ticks", 3),
        folder=tmp_path,
    )
    assert landed(tmp_path) == [(0, "down RIGHT"), (6, "up RIGHT")], "a tap between advances landed"
    assert (record["outcome"], record["ticks"]) == ("unfinished", 12)
    assert record["stopped"] == "agent-exited"
    assert "F1" in containers.task(images, "maze", ("ticks", 3))
    assert leftovers(images) == []


def pressing(times, gap):
    """An agent's command that presses F1 `times` times, `gap` seconds apart, then waits."""
    code = (
        "import time\n"
        "from Xlib import X, XK, display\n"
        "from Xlib.ext import xtest\n"
        "screen = display.Display()\n"
        "code = screen.keysym_to_keycode(XK.string_to_keysym('F1'))\n"
        f"for _ in range({times}):\n"
        "    xtest.fake_input(screen, X.KeyPress, code)\n"
        "    xtest.fake_input(screen, X.KeyRelease, code)\n"
        "    screen.sync()\n"
        f"    time.sleep({gap})\n"
        "time.sleep(3600)\n"
    )
    return f"python3 -c {shlex.quote(code)} {{task}}"


def test_a_game_in_ticks_that_ends_while_its_agent_plays_on_leaves_nothing_behind(images, tmp_path):
    record = containers.play(
        images, "minigames-agent", pressing(10**9, 0), "circles", pace=("ticks", 3),
        folder=tmp_path,
    )
    assert (record["outcome"], record["ticks"], record["stopped"]) == ("lost", 96, None)
    assert leftovers(images) == []


def test_the_agent_image_plays_with_xdotool_and_knows_the_screen_it_is_given(images, tmp_path):
    script = (
        "xdotool mousemove 100 100 keydown Right key F1 keyup Right key F1; "
        'xdpyinfo | grep dimensions; echo "$WIDTH $HEIGHT"'
    )
    containers.play(
        images, "minigames-agent", f"sh -c {shlex.quote(script)} {{task}}", "maze",
        pace=("ticks", 3), folder=tmp_path,
    )
    assert landed(tmp_path) == [(0, "move 100 100"), (0, "down RIGHT"), (3, "up RIGHT")]
    said = (tmp_path / "agent-stdout.txt").read_text()
    assert "640x480 pixels" in said and "640 480" in said, said
    assert leftovers(images) == []


def test_in_ticks_an_agent_that_stops_pressing_or_runs_too_long_is_stopped(images, tmp_path):
    idle = containers.play(
        images, "minigames-agent", pressing(3, 1), "maze", pace=("ticks", 3),
        folder=tmp_path / "idle", idle_timeout=2.5,
    )
    assert (idle["outcome"], idle["ticks"]) == ("unfinished", 9)
    assert idle["stopped"] == "idle-timeout"
    assert idle["seconds"] > 4, "a press of F1 did not put the idle timeout off"
    total = containers.play(
        images, "minigames-agent", pressing(10**9, 0.5), "maze", pace=("ticks", 3),
        folder=tmp_path / "total", idle_timeout=2.5, total_timeout=4,
    )
    assert (total["outcome"], total["stopped"]) == ("unfinished", "total-timeout")
    assert leftovers(images) == []


def test_a_run_that_keeps_going_writes_down_a_game_that_cannot_be_played_and_plays_on(
    images, tmp_path
):
    failed = {}
    results = containers.sweep(
        images, "minigames-test-bare", "anything {task}", ["circles", "dash"], out=tmp_path,
        failed=failed,
    )
    assert results == {} and list(failed) == ["circles", "dash"]
    assert "sleep" in (tmp_path / "circles" / "error.txt").read_text()
    assert leftovers(images) == []
