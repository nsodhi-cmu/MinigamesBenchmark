# Web

The crate `minigame-web` and the page beside it: the games in a browser. It is a front end to the engine's `Shell`, like the desktop window, and nothing the benchmark touches.

## The Crate

One `cdylib` for `wasm32-unknown-unknown`, holding a `Shell`, a `Frame`, an RGBA buffer, and the line it reads actions from. There is no binding library: it exports plain functions over numbers, and the page reads the module's memory directly.

| Export | What it does |
|---|---|
| `width`, `height` | The size of the picture |
| `start(seed)` | Opens the menu on a fresh shell |
| `line`, `line_length` | Where the page writes a line of the action language, and how long it may be |
| `act(length)` | Reads that many bytes of the line as one action of the action language and hands it to the shell. A line that does not parse is dropped. |
| `advance(seconds)` | That much of the player's time passing |
| `closed` | Whether escape has closed the shell |
| `draw` | Draws the screen and returns the address of its bytes |

## The Page

`index.html` is the whole of it: one canvas the size of the picture, scaled to twice its size where there is room, with pixels kept square.

- Keys go as `down` and `up` lines. Repeats of a held key are passed on.
- The pointer goes as `move`, `press`, and `release` in picture pixels.
- A frame is drawn on every animation frame, telling the shell how long it has been since the last.
- Escape from the menu closes the shell, and the page opens a fresh one.
- Keys held with control, alt, or the command key are left to the browser.
