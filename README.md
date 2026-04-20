# Idot

[![CMake Multi-Platform](https://github.com/fkm-X3/Idot/actions/workflows/cmake-multi-platform.yml/badge.svg)](https://github.com/fkm-X3/Idot/actions/workflows/cmake-multi-platform.yml)

Idot is a small interpreted language implemented in C++20.

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

### Configure

```powershell
cmake --preset x64-debug
```

### Build

```powershell
cmake --build --preset x64-debug
```

## Run

### Run a file

```powershell
.\out\build\x64-debug\Debug\Idot.exe .\examples\sample.idot
```

### Run REPL

```powershell
.\out\build\x64-debug\Debug\Idot.exe
```

Type `exit` or `quit` to stop the REPL.

### Run the Rust compiler

```powershell
.\out\build\x64-debug\cargo-target\debug\idotc.exe .\examples\sample.idot out.c
```

## Test

```powershell
ctest --preset x64-debug
```

The CMake build now covers both C++ (`Idot`, `idot_tests`) and Rust (`idotc`) targets, and the CTest preset runs both C++ and Rust tests.
