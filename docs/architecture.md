# Compiler Architecture

## Phases

| Phase | Module | Output |
|-------|--------|--------|
| Lexer | `idot::lexer` | `Vec<Token>` |
| Parser | `idot::parser` | `Vec<Decl>` (recursive descent) |
| Semantic | `idot::semantic` | Annotated AST (resolved names, types) |
| Codegen (C) | `idot::codegen::c` | C89 source string |

The compiler emits C89-compatible code and invokes the system C compiler
(`cc`/`clang`) for the `compile` and `run` commands.

## Roadmap

- **Part 1** (done): Lexer, parser, C backend, CLI, package manager
- **Part 2** (done): Semantic analyzer, type inference, type checking, struct codegen
- **Part 3** (done): Standard library, imports/resolution, enum/union codegen
- **Part 4** (in progress): Self-hosting — rewrite compiler in Idot

## Language Features

| Feature | Status |
|---------|--------|
| Primitives (i32, f64, etc.) | Done |
| Variables (let, let mut, const) | Done |
| Functions (fn, recursion) | Done |
| Control flow (if/else, while, for, match) | Done |
| Structs with field access | Done |
| Enums / Unions | In progress |
| Generics | Done |
| Pointers | Done |
| Defer | Done |
| Compile-time evaluation | Done |
| Standard library | Done |
| Imports / module resolution | Done |
| Self-hosting | In progress |
