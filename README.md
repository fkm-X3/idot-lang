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
cmake -S . -B out\build\vs-debug -G "Visual Studio 17 2022" -A x64
```

### Build

```powershell
cmake --build out\build\vs-debug --config Debug
```

## Run

### Run a file

```powershell
.\out\build\vs-debug\Debug\Idot.exe .\examples\sample.idot
```

### Run REPL

```powershell
.\out\build\vs-debug\Debug\Idot.exe
```

Type `exit` or `quit` to stop the REPL.

## Test

```powershell
ctest --test-dir out\build\vs-debug -C Debug --output-on-failure
```
