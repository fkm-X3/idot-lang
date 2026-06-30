# Idot

A low-level systems language prototyped in Rust. Idot compiles to C89 and
leverages the system C compiler for code generation. The project is building
toward self-hosting.

<!-- LANG_BAR_START -->
### Project Language Breakdown

<table>
  <tr>
    <td width="56%" bgcolor="#8510d8">&nbsp;</td>
    <td width="44%" bgcolor="#dea584">&nbsp;</td>
  </tr>
  <tr>
    <td align="center"><b>Idot</b> 56%</td>
    <td align="center"><b>Rust</b> 44%</td>
  </tr>
</table>

<!-- LANG_BAR_END -->

```bash
# Build the compiler
cd compiler && cargo build

# Compile and run (one step)
cd compiler && cargo run -- run ../examples/fib.ido

# Compile to C, then compile with clang/cc
cd compiler && cargo run -- compile ../examples/hello.ido
clang -o hello ../examples/hello.ido.c && ./hello

# Using the package manager
cd matrix && cargo run -- new my_project
cd my_project && matrix build && matrix run
```

## Examples

| File | Compiles | Runs | Shows |
|------|----------|------|-------|
| `hello.ido` | yes | exit 0 | Minimal program |
| `fib.ido` | yes | exit 55 | Functions, recursion |
| `math.ido` | yes | exit 61 | Arithmetic, function composition |
| `control_flow.ido` | yes | exit 333 | if/else |
| `types.ido` | yes | exit 181 | Variables, type inference, assignment |
| `structs.ido` | yes | exit 32 | Struct types, field access, literals |
| `for_loop.ido` | yes | exit 10 | while loops |

## Language Tour

### Variables

```idot
let x: i32 = 0;          // explicit type, immutable by default
let y = 0;                // inferred type, immutable
let mut z: i32 = 0;       // mutable, explicit type
let mut w = 0;            // mutable, inferred type
const pi: f64 = 3.14159;  // compile-time constant
```

All variables must be initialized. Mutability is opt-in via `let mut`.
Constants are declared with `const` and must be evaluable at compile time.

### Functions

```idot
fn add(x: i32, y: i32) -> i32 {
    return x + y;
}

fn main() -> i32 {
    let result = add(3, 4);
    return result;
}
```

Functions are declared with `fn`. Parameters are typed. Return types are
specified with `->`. Functions without a return arrow implicitly return void.

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
    let mut i = 0;
    let mut acc = 0;
    while limit > i {
        acc = acc + i;
        i = i + 1;
    }
    return acc;
}
```

`if` / `else` and `while` are expressions. `for..in` loops iterate over
containers. `match` provides pattern matching with wildcard fallthrough.

### Structs

```idot
struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}

fn dot(a: Vec3, b: Vec3) -> i32 {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}
```

Struct literals use named fields: `Vec3{ x = 1, y = 2, z = 3 }`.

## Compiler Architecture

| Phase | Module | Output |
|-------|--------|--------|
| Lexer | `idot::lexer` | `Vec<Token>` |
| Parser | `idot::parser` | `Vec<Decl>` (recursive descent) |
| Semantic | `idot::semantic` | Annotated AST (resolved names, types) |
| Codegen (C) | `idot::codegen::c` | C89 source string |

The compiler emits C89-compatible code and invokes the system C compiler
(`cc`/`clang`) for the `compile` and `run` commands.

## CLI

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
    Cargo.toml
  matrix/                # Package manager (Rust)
    src/
      commands.rs        # new, build, run, test
      manifest.rs        # matrix.toml parser
      main.rs
    Cargo.toml
  examples/              # Example Idot programs
  README.md
```

## Status

Working prototype. The compiler lexes, parses, semantically analyzes,
and emits working C code for a substantial subset of the language:
primitives, variables, functions, control flow (`if`/`while`/`for`/`match`),
and user-defined struct types with field access and compound literals.
Enum and union codegen are in progress; the package manager supports
`new`, `build`, `run`, and `test` commands.

## Roadmap

- **Part 1** (done): Lexer, parser, C backend, CLI, package manager
- **Part 2** (done): Semantic analyzer, type inference, type checking, struct codegen
- **Part 3** (done): Standard library, imports/resolution, enum/union codegen
- **Part 4** (in progress): Self-hosting — rewrite compiler in Idot

## License

[Apache 2.0](LICENSE)
