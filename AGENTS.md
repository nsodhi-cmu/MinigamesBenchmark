# Repository Structure

## Documentation Structure

- All documentation belongs inside of `docs/`. There should **NEVER** be any stray markdown files or documentation outside of this folder. The **ONLY** exceptions are the root level `README.md`, this `AGENTS.md` file, and skills under `.agents/skills/`.
- Each component of the repository has one file in `docs/`, named for the component with the relevant information about it.
- To get oriented, start from `README.md` (which contains a repository-wide overview), then `docs/`.
- Every markdown file is a factual description of the repository as it is, written so that a newcomer can read it and relate it directly to the code. If a sentence would not be true of the code as it stands, it does not belong.
- Update a component's doc in the same change that alters the code it describes.
- Do not write counts, ranges, or limits into documentation unless the code enforces them. A number that is merely what happens to be true today reads as a rule to the next reader.

## Code Structure

- One workspace. `README.md` documents the components and the build and test commands; keep it accurate rather than duplicating it here.
- Games are written in Rust and drawn by the crate itself. No game engine, no GPU, no windowing library inside a game.

## Games

- The frame is the interface, and nothing about a game's state reaches a player any other way: no accessibility tree, no object list. Anything a player must know is drawn by the game itself, with nothing laid over or around it.
- No game may carry the name, likeness, characters, artwork, or sounds of anything published by someone else. A game is described by what it asks of the player.
- A game is finished or it is not, and is scored on finishing or on time: no par, no reference time, no partial credit.
- A game reads `Input` and writes into a `Frame`. It reads no clock, draws no window, and touches no file.

# Style

## Comments

- Comments say what a thing is, not why it came to be; a rationale for a larger decision belongs in `docs/`.
- Comment sparingly: never restate the code, and almost never comment inside a function body.
- No comment or doc names a person, conversation, past attempt, issue number, or commit hash.

## Rust

- Tests live beside the source in a mirrored tree, not inline:
  ```
  src/frame.rs   →  src/tests/frame.rs
  ```
  wired at the bottom of the module:
  ```rust
  #[cfg(test)]
  #[path = "tests/frame.rs"]
  mod tests;
  ```
- `rustfmt.toml` at the repo root governs everything. Run `cargo fmt` before committing.
- `cargo clippy --all-targets -- -D warnings` must be clean. Fix a lint by default. The one exception is where fixing it makes the code more complicated: then use `#[allow]` scoped to the smallest span that works, with a one-line comment naming what it would break.

## Python

- The Python side is the benchmark harness and nothing else. Rules of play live in Rust.
- Keep it to the standard library where it can.

## Tests

- Test names are sentences describing the claim: `a_tap_is_over_by_the_time_the_tick_settles`, not `test_input_2`.
- Keep the set small and keep it hard. A test that is a tautology by the way the code is constructed should not exist; a test that would catch a subtle regression should.
- Tests check objective truths, never difficulty or level design; difficulty is measured by a separate script.
- A new game is not finished until it survives the catalog tests: playing it badly for its whole limit, and repeating itself exactly from a seed and a script.

## Logging

- There is no logging code unless a feature asks for it. Where there is logging, it is toggled only by a command-line boolean that is off by default, never by an environment variable.

# Prioritizations

- Write simple code that reads as a plain description of what it does: prefer the obvious approach over the clever one. Fewer lines is a soft priority.
- Prefer OS-agnostic code; where a platform split is unavoidable, isolate it behind one interface rather than branching inline.
- Avoid hyperparameters, config flags, and knobs; `seed` and the pace are the only things that shape an episode.
- Prefer making illegal states unrepresentable over documenting that they're illegal.
- Prefer lightweight, portable, well-maintained dependencies.

# Git

- **NEVER** put a session link, a conversation URL, or any other pointer back to the tool that wrote the code into a commit message or a pull request description. A co-author trailer is fine; a link is not.
