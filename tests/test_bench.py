"""What the Python side promises: a picture, an action, and a run's plan and score."""

import pytest

import minigames
from minigames.bench import runner, scoring


def test_a_frame_is_a_png_of_the_size_the_module_reports():
    episode = minigames.Episode("circles", seed=1)
    assert episode.frame()[1:4] == b"PNG"
    assert len(episode.pixels()) == minigames.size()[0] * minigames.size()[1] * 3


def test_an_action_that_does_not_parse_is_refused_rather_than_ignored():
    with pytest.raises(ValueError, match="teleport"):
        minigames.Episode("serpent", seed=1).act("teleport 3 4")


def test_only_a_win_passes_and_a_game_stopped_or_never_played_is_incomplete():
    results = {
        "maze": {"outcome": "won", "ticks": 240},
        "eye": {"outcome": "lost", "ticks": 10800},
        "keys": {"outcome": "unfinished", "ticks": 90},
    }
    summary = scoring.summarize(["maze", "eye", "keys", "dash"], results, "2026-10-04T00:00:00+00:00")
    assert (summary["passed"], summary["failed"]) == (["maze"], ["eye"])
    assert summary["incomplete"] == ["keys", "dash"]
    assert (summary["score"], summary["total"]) == (1, 4)


def test_a_run_plays_each_game_named_once_or_every_game():
    assert runner.chosen(["balance", "balance", "eye"]) == ["balance", "eye"]
    assert runner.chosen() == list(runner.specs())
    memory = runner.tagged(["memory"])
    assert memory and all("memory" in runner.specs()[game]["tags"] for game in memory)
    with pytest.raises(ValueError, match="nosuchgame"):
        runner.chosen(["serpent", "nosuchgame"])


def test_a_run_names_its_agent_its_engine_and_one_pace(monkeypatch):
    from minigames.bench.__main__ import main

    agent = ["--run-agent", "image", "agent {task}"]
    for words in (
        ["--engine", "podman"],
        agent,
        [*agent, "--engine", "lxc"],
        [*agent, "--engine", "docker", "--ticks", "3", "--clock", "6"],
    ):
        monkeypatch.setattr("sys.argv", ["bench", *words])
        with pytest.raises(SystemExit) as refused:
            main()
        assert refused.value.code == 2, words
    for words, said in (
        ([*agent, "--engine", "podman", "--ticks", "0"], "--ticks is from 1"),
        ([*agent, "--engine", "podman", "--clock", "61"], "--clock is from 1"),
        ([*agent, "--engine", "podman", "--total-timeout", "60"], "takes no timeout"),
        ([*agent, "--engine", "podman", "--ticks", "--idle-timeout", "0"], "positive number"),
        (["--run-agent", "image", "agent", "--engine", "podman"], "no {task}"),
    ):
        monkeypatch.setattr("sys.argv", ["bench", *words])
        with pytest.raises(SystemExit, match=said):
            main()
