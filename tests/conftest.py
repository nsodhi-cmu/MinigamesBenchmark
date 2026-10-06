"""`--containers ENGINE` also plays episodes in containers on that engine, building every image
they need first."""

import subprocess
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parent.parent


def pytest_addoption(parser):
    parser.addoption(
        "--containers",
        choices=("podman", "docker"),
        help="also play episodes in containers on this engine",
    )


def pytest_collection_modifyitems(config, items):
    if config.getoption("--containers"):
        return
    for item in items:
        if "containers" in item.keywords:
            item.add_marker(pytest.mark.skip(reason="plays in containers; run with --containers"))


@pytest.fixture(scope="session")
def images(pytestconfig):
    """The engine named by --containers, once it has the game image, the agent image, and one
    image for each agent under tests/agents."""
    from minigames.bench.containers import installed

    runs = pytestconfig.getoption("--containers")
    installed(runs)

    def build(tag, folder, *more):
        file = str(folder / "Containerfile")
        done = subprocess.run(
            [runs, "build", "-q", "-f", file, "-t", tag, *more, str(folder)],
            capture_output=True,
            text=True,
        )
        assert done.returncode == 0, f"{tag} did not build: {done.stderr}"

    build("minigames-game", ROOT, "--target", "game")
    build("minigames-agent", ROOT / "agent")
    for folder in sorted((ROOT / "tests" / "agents").iterdir()):
        build(f"minigames-test-{folder.name}", folder)
    return runs
