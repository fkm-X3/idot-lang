# Idot Language

A low-level systems language blending Zig + Odin syntax and philosophy.
Prototyped in Rust, targeting self-hosting.

Package manager: matrix (analogous to cargo)

## Quick Start

```bash
# Build the compiler
cd compiler && cargo build

# Compile and run (one step)
cd compiler && cargo run -- run ../examples/fib.ido
echo $?   # prints 55

# Compile to C, then compile with clang/cc
cd compiler && cargo run -- compile ../examples/hello.ido
clang -o hello ../examples/hello.ido.c && ./hello

# Using the package manager
cd matrix && cargo run -- new my_project
cd my_project && matrix build && matrix run
```

## Examples

| File | Compiles | Runs | Shows |
|---|---|---|---|
| `hello.ido` | yes | exit 0 | Minimal program |
| `fib.ido` | yes | exit 55 | Functions, recursion, const inference |
| `math.ido` | yes | exit 61 | Arithmetic, function composition |
| `control_flow.ido` | yes | exit 333 | if/else, while loops |
| `types.ido` | yes | exit 181 | Variables, type inference, assignment |
| `structs.ido` | yes* | codegen pending | Struct types, field access, literals |

\* structs.ido parses and passes semantic analysis; C codegen for user-defined
  struct types is not yet implemented (the C backend currently only handles
  built-in scalar types).

## Language Tour

### Variables

```idot
x: i32 = 0;           // mutable, explicit type (Odin-style)
y := 0;               // mutable, inferred type
const z: i32 = 0;     // immutable, explicit type (Zig-style)
const w := 0;         // immutable, inferred type
pi :: 3.14159;        // Odin shorthand: typed constant
```

All variables must be initialized. The semantic analyzer infers types
from initializers and reports mismatches.

### Functions

```idot
fn add(x: i32, y: i32) -> i32 {
    return x + y;
}

fn main() -> i32 {
    const result := add(3, 4);
    return result;  // 7
}
```

### Control Flow

```idot
fn classify(x: i32) -> i32 {
    if x < 0 {
        return 111;
    } else {
        return 222;
    }
}

fn sum_to(limit: i32) -> i32 {
    var i: i32 = 0;
    var acc: i32 = 0;
    while limit > i {
        acc = acc + i;
        i = i + 1;
    }
    return acc;
}
```

### Composite Types

```idot
Vec3 :: struct {
    x: i32,
    y: i32,
    z: i32,
};

fn dot(a: Vec3, b: Vec3) -> i32 {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}
```

## Compiler Architecture

| Phase | Module | Output |
|---|---|---|
| Lexer | `idot::lexer` | `Vec<Token>` |
| Parser | `idot::parser` | `Vec<Decl>` (recursive descent) |
| Semantic | `idot::semantic` | Annotated AST (resolved names, types) |
| Codegen (C) | `idot::codegen::c` | C source string |

The compiler emits C89-compatible code and invokes the system C compiler
(`cc`/`clang`) for the `compile` and `run` commands.

## CLI

```bash
idot compile <file>     # compile → .c → .exe
idot compile <file> --emit-c  # emit C only, don't invoke cc
idot run <file>         # compile + execute

matrix new <name>       # scaffold a new project
matrix build            # build project in current dir
matrix run              # build + run
matrix test             # run tests
```

## Project Structure

```
idot/
├── compiler/              # The Idot compiler (Rust)
│   ├── src/
│   │   ├── lexer.rs       # Tokenizer
│   │   ├── parser.rs      # Recursive-descent parser
│   │   ├── ast.rs         # AST node definitions + TypeVal
│   │   ├── semantic.rs    # Name resolution + type checking
│   │   ├── codegen/c.rs   # C backend
│   │   ├── lib.rs         # Library entry point
│   │   └── main.rs        # CLI
│   └── Cargo.toml
├── matrix/                # Package manager (Rust)
│   ├── src/
│   │   ├── commands.rs    # new, build, run, test
│   │   ├── manifest.rs    # matrix.toml parser
│   │   └── main.rs
│   └── Cargo.toml
├── examples/              # Example Idot programs
└── README.md              # This file
```

## Status

Early prototype. The compiler lexes, parses, semantically analyzes,
and emits C code for a substantial subset of the language. The C backend is
functional for scalar types, arithmetic, control flow, and functions; struct,
enum, and union codegen are still in progress.

## Roadmap

- **Part 1** (done): Lexer, parser, C backend, CLI, package manager
- **Part 2** (in progress): Semantic analyzer, type inference, type checking
- **Part 3**: Standard library, imports/resolution, more codegen features
- **Part 4**: Self-hosting — rewrite compiler in Idot

## License

MIT
