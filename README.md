# Matrix Rain (Rust + Crossterm)

A cinematic Matrix-style terminal rain effect built in Rust using `crossterm` and `rand`.

## Features

- Full-screen terminal rain animation
- Per-column randomized speed, trail length, and restart delay
- Character mutation while falling
- Truecolor glow gradient:
  - head: bold white
  - second glyph: bright green
  - trail: medium green to near-black green fade
- Resize-aware rendering
- Non-blocking input and graceful exit (`q` or `Ctrl+C`)
- Diff-based renderer (updates changed cells only)

## Tech Stack

- Rust (binary crate)
- `crossterm = "0.27"`
- `rand = "0.8"`

## Project Structure

```text
src/
  main.rs      # app bootstrap, terminal guard, frame loop
  column.rs    # Column state + update/mutation logic
  renderer.rs  # RGB coloring + diff rendering
  input.rs     # non-blocking key/resize polling
```

## Run Locally

```bash
cargo run
```

## Controls

- `q` -> quit
- `Ctrl+C` -> quit

## Customize Glyph Style

In `src/column.rs`, change:

```rust
const GLYPH_STYLE: GlyphStyle = GlyphStyle::Balanced;
```

Available modes:

- `GlyphStyle::ClassicMatrix`
- `GlyphStyle::Balanced`
- `GlyphStyle::AsciiGlitch`

## Build Check

```bash
cargo check
```

## Notes

- Best viewed in a terminal with truecolor support.
- If your terminal font lacks some glyphs, switch to `GlyphStyle::AsciiGlitch`.
