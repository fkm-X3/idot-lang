# Idot

[![Rust Multi-Platform](https://github.com/fkm-X3/Idot/actions/workflows/rust-multi-platform.yml/badge.svg)](https://github.com/fkm-X3/Idot/actions/workflows/rust-multi-platform.yml)

Idot is a small compiled language implemented in Rust with a growing set of features.

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

### Compilation Model

Idot has transitioned from JIT interpretation to ahead-of-time (AOT) compilation:

- **AOT Compiler**: The `idot` binary now compiles Idot source code to COFF object files using Cranelift's ObjectModule backend
- **Runtime Library**: A separate `idot-runtime` crate provides the runtime functions required by compiled code
- **Object Files**: Generated `.o` files can be linked with the runtime library to create standalone executables

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

### Compile to object file

The `idot` binary compiles Idot source to COFF object files:

```powershell
cargo run -p idot --bin idot -- .\examples\sample.idot
```

This generates `.\examples\sample.idot.o` which can be linked with the runtime library to create an executable.

### Run the alternative native backend

The `idotc` binary provides the previous JIT-based execution model for comparison:

```powershell
cargo run -p idot --bin idotc -- .\examples\sample.idot
```

`idotc` compiles Idot source directly to machine code via Cranelift's JIT module and executes it immediately.

## Test

```powershell
cargo test --workspace
```

### Graphics tests

```powershell
python test_graphics.py
```

## Architecture

### Compilation Pipeline

1. **Lexer**: Tokenizes source code
2. **Parser**: Builds an abstract syntax tree (AST)
3. **Code Generation**: Converts AST to Cranelift IR
4. **Object Emission**: ObjectModule emits COFF object files
5. **Linking**: Object files are linked with `idot-runtime` to create executables

### Crates

- `idot` (compiler): Main compiler and code generation
- `idot-graphics`: Graphics library bindings
- `idot-runtime`: Runtime functions for compiled code (values, operators, I/O)
