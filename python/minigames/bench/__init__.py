"""Running agents in containers against the games, and sorting how the games ended."""

from .containers import play, sweep
from .scoring import summarize

__all__ = ["play", "summarize", "sweep"]
