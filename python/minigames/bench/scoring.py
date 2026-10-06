"""A run's games sorted by how they ended."""

from __future__ import annotations


def summarize(games: list[str], results: dict[str, dict], ended: str) -> dict:
    """`games` as passed, failed, or neither, from the `results` there are for them. A game with
    no result, or one stopped before it ended, is incomplete."""
    passed = [game for game in games if results.get(game, {}).get("outcome") == "won"]
    failed = [game for game in games if results.get(game, {}).get("outcome") == "lost"]
    incomplete = [game for game in games if game not in passed and game not in failed]
    return {
        "ended": ended,
        "passed": passed,
        "failed": failed,
        "incomplete": incomplete,
        "score": len(passed),
        "total": len(games),
    }
