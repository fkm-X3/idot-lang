# Idot

[![Rust Multi-Platform](https://github.com/fkm-X3/Idot/actions/workflows/rust-multi-platform.yml/badge.svg)](https://github.com/fkm-X3/Idot/actions/workflows/rust-multi-platform.yml)

Idot is a small interpreted language implemented in Rust with a growing set of features.

## Features

### Language Features (MVP)

The current MVP includes:

- expressions (arithmetic, comparison, equality, unary operators)
- variable declarations and assignment (`let x = ...;`, `x = ...;`)
- conditionals (`if (...) ... else ...`)
- block scope (`{ ... }`)
- `print` statements for output
- **Function calls** (new!)

Loops and user-defined functions are deferred to post-MVP milestones.

### Graphics Library (New!)

Idot now includes a built-in graphics library with support for:

- Window creation and management
- Shape drawing (rectangles, circles, lines)
- Full hex color support
- SVG rendering output

See [GRAPHICS_GUIDE.md](GRAPHICS_GUIDE.md) for detailed documentation.

## Syntax example

```idot
let x = 3;
if (x > 2) {
  print "big";
} else {
  print "small";
}
print x + 1;
```

Expected output:

```text
big
4
```

## Graphics example

```idot
create_window(800, 600);
draw_rect(100, 100, 200, 150, "#FF0000");
draw_circle(400, 300, 75, "#00FF00");
draw_line(0, 0, 800, 600, "#0000FF");
```

## Build

```powershell
cargo build --workspace
```

## Run

### Run a file

```powershell
cargo run -p idot --bin idot -- .\examples\sample.idot
```

### Run graphics example

```powershell
cargo run -p idot --bin idot -- .\examples\graphics_demo.idot
```

### Run REPL

```powershell
cargo run -p idot --bin idot
```

Type `exit` or `quit` to stop the REPL.

### Run the native compiler backend

```powershell
cargo run -p idot --bin idotc -- .\examples\sample.idot
```

`idotc` compiles Idot source directly to machine code via a native backend and executes it.

## Test

```powershell
cargo test --workspace
```

### Graphics tests

```powershell
python test_graphics.py
```

