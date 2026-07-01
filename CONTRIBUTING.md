# Contributing to Idot

Thanks for your interest in contributing to Idot! Idot is a low-level systems
language prototyped in Rust that compiles to C89 and is working toward self-hosting.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- A C compiler (`cc` / `clang` / `gcc`) — the system one is used for codegen
- (Windows) Visual Studio Build Tools or LLVM/Clang

### Building

```bash
# Build the Rust bootstrap compiler
cd compiler && cargo build

# Or build everything (compiler + package manager)
cargo build --workspace
```

### Running

```bash
# Compile and run an Idot program in one step
cd compiler && cargo run -- run ../examples/hello.ido

# Compile to C only
cd compiler && cargo run -- compile ../examples/hello.ido

# Compile to C then to a binary (system cc invoked automatically)
cd compiler && cargo run -- compile ../examples/hello.ido
./build/hello          # or hello.exe on Windows
```

### Testing

```bash
# Test standard library modules
cargo run --bin idot compile test-std-lib/src/test_mem.ido && ./build/test_mem
cargo run --bin idot compile test-std-lib/src/test_math.ido && ./build/test_math
cargo run --bin idot compile test-std-lib/src/test_collections.ido && ./build/test_collections
cargo run --bin idot compile test-std-lib/src/test_c.ido && ./build/test_c
cargo run --bin idot compile test-std-lib/src/test_io.ido && ./build/test_io

# Test compiler features (generics, imports)
cargo run --bin idot compile tests/test_generics.ido && ./build/test_generics
cargo run --bin idot compile tests/test_imports.ido && ./build/test_imports

# Self-hosting test
cargo run --bin idot compile compiler/src/main.ido

# Clean up test artifacts
rm -f build/test_*.c build/test_*.exe build/test_*.out
```

### Using the Package Manager

```bash
cd matrix && cargo run -- new my_project
cd my_project && matrix build && matrix run
```

## How to Claim Your Early Adopter Slot

We are giving away 200 permanent slots to the first unique repositories built
using our language! To claim your slot, follow these steps:

1. Create a public repository containing your unique code.
2. Fork this repository.
3. Open `data/early_adopters.json` in your fork and append your project to the
   array:

```json
{
  "github_username": "your-username",
  "repo_url": "https://github.com/your-username/your-repo",
  "project_description": "A short 1-sentence description of what you built."
}
```

4. Submit a Pull Request. Our automated workflow will check your submission and
   update the `EARLY_ADOPTERS.md` file automatically!

## Project Structure

```
idot/
  compiler/              # The Idot compiler (Rust)
    src/
      lexer.rs           # Tokenizer
      parser.rs          # Recursive-descent parser
      ast.rs             # AST node definitions
      semantic.rs        # Name resolution + type checking
      codegen/
        c.rs             # C89 backend
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
```

## Making Changes

### Commits

- Use clear, descriptive commit messages.
- Keep commits focused on a single logical change.
- Reference issues or pull requests when relevant.

### Code Style

- **Rust code:** Follow standard Rust conventions (`cargo fmt`, `cargo clippy`).
- **Idot code:** Follow the patterns in existing `.ido` files — the language is
  evolving, so consistency with the existing codebase is the best guide.
- **C codegen output:** Generated C should be C89-compatible. Avoid
  C99/C11 features unless unavoidable.

### Important: Matrix Bootstrap vs. Idot Source of Truth

The `matrix/` crate is a **bootstrap copy** of the Idot build/test toolchain.
Do **not** edit `matrix/src/commands.rs` directly unless syncing changes.
Instead:

1. Edit the canonical Idot version in `compiler/src/cmd_*.ido`
2. Optionally sync the changes to `matrix/` to keep the bootstrap working

The Idot implementations in `compiler/src/cmd_*.ido` are the source of truth.

## Pull Request Process

1. **Open an issue** first for significant changes (new features, breaking
   changes, large refactors) to discuss the approach.
2. **Fork the repo** and create a feature branch from `main`.
3. **Make your changes** following the code style and conventions above.
4. **Test your changes** — ensure existing tests pass and add new ones for
   new functionality.
5. **Run the self-hosting test** (`cargo run --bin idot compile compiler/src/main.ido`)
   to verify the compiler can compile itself.
6. **Open a pull request** against `main` with a clear title and description.

### PR Checklist

- [ ] Builds without warnings (`cargo build --workspace`)
- [ ] Existing tests pass
- [ ] New tests added for new functionality
- [ ] Self-hosting test passes
- [ ] Commits are clean and descriptive

## Reporting Issues

- Check existing issues to avoid duplicates.
- Include the Idot version or commit hash.
- Provide a minimal reproduction case (a short `.ido` file if possible).
- Describe expected vs. actual behavior.
- Include your OS and C compiler version if relevant.

## License

By contributing, you agree that your contributions will be licensed under the
[Apache License 2.0](LICENSE), the same license as the project.
