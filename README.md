# idot-lang

A minimal programming language that compiles via LLVM IR and `clang`.

## Example

```
fn main() -> int {
    let x: int = 10;
    let y: int = 20;
    let z: int = x + y * 2;
    print(z);

    if z > 40 {
        print(1);
    } else {
        print(0);
    }

    let i: int = 0;
    while i < 5 {
        print(i);
        i = i + 1;
    }
}
```

## Build & Run

```
cargo build
cargo run -- program.idot
```

The compiler pipelines source → lexer → parser → LLVM IR text → `clang` → native executable.

## Language

- Types: `int` (64-bit signed integer), `bool` (0/1)
- Variables: `let name: type = expr;`
- Assign: `name = expr;`
- Arithmetic: `+`, `-`, `*`, `/`
- Comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Control flow: `if/else`, `while`
- Blocks: `{ ... }`
- Built-in: `print(expr)` outputs to stdout
- Comments: `// line comments`

Every function returns `i64`. Implicit `return 0` at the end of `main`.

## Implementation

- **Lexer** — tokenizes source into keyword, identifier, number, and operator tokens
- **Parser** — recursive descent, precedence climbing for expressions, produces AST
- **Codegen** — generates human-readable LLVM IR (`.ll`) instead of using inkwell/llvm-sys
- **Compilation** — emits IR to a temp file and invokes `clang` to produce an executable

Requires [LLVM/clang](https://llvm.org/) on `PATH`.
