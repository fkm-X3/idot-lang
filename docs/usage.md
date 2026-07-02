# CLI Reference & Project Structure

## Commands

```bash
idot compile <file>           # compile -> .c -> .exe
idot compile <file> --emit-c  # emit C only, don't invoke cc
idot run <file>               # compile + execute

matrix new <name>             # scaffold a new project
matrix build                  # build project in current dir
matrix run                    # build + run
matrix test                   # run tests
```

## Project Structure

```
idot/
  compiler/              # The Idot compiler (Rust)
    src/
      lexer.rs           # Tokenizer
      parser.rs          # Recursive-descent parser
      ast.rs             # AST node definitions + TypeVal
      semantic.rs        # Name resolution + type checking
      codegen/
        c.rs             # C backend
      lib.rs             # Library entry point
      main.rs            # CLI
      cmd_*.ido          # Self-hosted command implementations
    Cargo.toml
  matrix/                # Package manager bootstrap (Rust)
    src/
      commands.rs        # new, build, run, test
      manifest.rs        # matrix.toml parser
      main.rs
    Cargo.toml
  lib/                   # Standard library (Idot)
    std.ido
    std/
      c.ido              # C bindings
      collections.ido    # Collection types
      io.ido             # I/O
      math.ido           # Math
      mem.ido            # Memory
  examples/              # Example Idot programs
  tests/                 # Compiler tests
  test-std-lib/          # Standard library tests
  build/                 # Generated C and binaries (gitignored)
  data/                  # Supporting data (e.g. early adopters registry)
```

## Important Note

The `matrix/` crate is a **bootstrap copy** of the Idot build/test toolchain.
The source of truth lives in `compiler/src/cmd_*.ido`. When contributing,
edit the Idot versions, not the Rust bootstrap directly.
