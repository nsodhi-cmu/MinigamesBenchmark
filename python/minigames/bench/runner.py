"""What a run plays and in what order, and how an episode is filmed."""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Callable, Iterable

import minigames


def specs() -> dict[str, dict]:
    return {spec["name"]: spec for spec in minigames.games()}


def chosen(games: Iterable[str] | None = None) -> list[str]:
    """Every game named, each once, or every game there is where none are."""
    known = specs()
    picked = list(dict.fromkeys(games)) if games else list(known)
    unknown = [game for game in picked if game not in known]
    if unknown:
        raise ValueError(f"no game called {', '.join(unknown)}. Known games: {', '.join(known)}")
    return picked


def tagged(tags: Iterable[str]) -> list[str]:
    """The games that carry any of `tags`."""
    wanted = set(tags)
    known = {tag for spec in minigames.games() for tag in spec["tags"]}
    if wanted - known:
        unknown = ", ".join(sorted(wanted - known))
        raise ValueError(f"no tag called {unknown}. Known tags: {', '.join(sorted(known))}")
    return [spec["name"] for spec in minigames.games() if wanted & set(spec["tags"])]


def encode(folder: Path, play: Callable[[Callable[[bytes], object]], object]) -> object:
    """`recording.mp4` in `folder`, from every frame `play` writes, a frame a tick at the speed
    the games are written to. What `play` answers is the answer."""
    import imageio_ffmpeg

    width, height = minigames.size()
    video = folder / "recording.mp4"
    encoder = subprocess.Popen(
        [
            imageio_ffmpeg.get_ffmpeg_exe(),
            *("-loglevel", "error", "-y"),
            *("-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{width}x{height}"),
            *("-r", str(minigames.tick_hz()), "-i", "-"),
            *("-c:v", "libx264", "-preset", "veryfast", "-threads", "1", "-pix_fmt", "yuv420p"),
            *("-movflags", "+faststart", str(video)),
        ],
        stdin=subprocess.PIPE,
    )
    try:
        played = play(encoder.stdin.write)
    finally:
        encoder.stdin.close()
        encoder.wait()
    if encoder.returncode != 0:
        raise RuntimeError(f"ffmpeg could not write {video}")
    return played
