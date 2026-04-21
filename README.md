# Idot

[![Rust Multi-Platform](https://github.com/fkm-X3/Idot/actions/workflows/rust-multi-platform.yml/badge.svg)](https://github.com/fkm-X3/Idot/actions/workflows/rust-multi-platform.yml)

Idot is a small interpreted language implemented in Rust.

## MVP scope

The current MVP is intentionally focused on:

- expressions (arithmetic, comparison, equality, unary operators)
- variable declarations and assignment (`let x = ...;`, `x = ...;`)
- conditionals (`if (...) ... else ...`)
- block scope (`{ ... }`)
- `print` statements for output

Loops and user-defined functions are deferred to post-MVP milestones.

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

## Build

```powershell
cargo build --workspace
```

## Run

### Run a file

```powershell
cargo run -p idot --bin idot -- .\examples\sample.idot
```

### Run REPL

```powershell
cargo run -p idot --bin idot
```

Type `exit` or `quit` to stop the REPL.

### Run the Rust compiler

```powershell
cargo run -p idot --bin idotc -- .\examples\sample.idot out.c
```

## Test

```powershell
cargo test --workspace
```
