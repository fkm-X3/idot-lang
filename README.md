# Idot

A low-level systems language prototyped in Rust. Idot compiles to C89 and
leverages the system C compiler for code generation. The project is building
toward self-hosting.

<!-- LANG_BAR_START -->
<p align="center">
  <img src="https://raw.githubusercontent.com/fkm-X3/idot-lang/refs/heads/assets/assets/lang-bar.svg" alt="Language breakdown">
</p>
<!-- LANG_BAR_END -->

## Prerequisites

- **Rust** (stable toolchain) — [rustup.rs](https://rustup.rs)
- **C compiler** — `cc` / `clang` / `gcc` (any will do)
- **Windows only:** Visual Studio Build Tools or LLVM/Clang

## Install

### Via script
```bash
# Linux / Mac
curl -fsSL https://fkm-X3.github.io/idot-lang/install.sh | sh
```
```powershell
# Windows
irm https://fkm-X3.github.io/idot-lang/install.ps1 | iex
```

### From source
```bash
git clone https://github.com/fkm-X3/idot-lang
cd idot-lang/compiler/self
cargo build --release
```
The binary is at `compiler/target/release/idot`.

## Usage

### Compile & run a single file
```bash
# One-shot (builds compiler on first run)
cd compiler && cargo run -- run ../examples/hello.ido

# Or with the installed binary
idot run examples/hello.ido
```

### Compile to C only
```bash
cd compiler && cargo run -- compile ../examples/hello.ido --emit-c
# Writes examples/hello.ido.c
```

### Full compilation pipeline (Idot → C → binary)
```bash
cd compiler && cargo run -- compile ../examples/hello.ido
./build/hello          # or hello.exe on Windows
```

### Using the package manager
```bash
cd matrix && cargo run -- new my_project
cd my_project && matrix build && matrix run
```

## Hello, World

```idot
// hello.ido
fn main() -> i32 {
    io::println("Hello, World!");
    return 0;
}
```

## Documentation

| Document | Description |
|----------|-------------|
| [Language Tour](docs/language-tour.md) | Variables, functions, control flow, structs, and more |
| [CLI Reference & Structure](docs/usage.md) | Commands, project layout |
| [Compiler Architecture](docs/architecture.md) | Phases, roadmap |
| [Contributing](CONTRIBUTING.md) | Building, testing, PR process |
| [Early Adopters](EARLY_ADOPTERS.md) | Claim your slot in the Hall of Fame |

## Status

Working prototype. The compiler lexes, parses, semantically analyzes,
and emits working C code for a substantial subset of the language:
primitives, variables, functions, control flow (`if`/`while`/`for`/`match`),
and user-defined struct types with field access and compound literals.
Enum and union codegen are in progress; the package manager supports
`new`, `build`, `run`, and `test` commands.

## License

[Apache 2.0](LICENSE)
