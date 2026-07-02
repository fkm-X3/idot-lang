# Language Tour

> See the [examples directory](../examples/) for runnable Idot programs.

## Variables

```idot
let x: i32 = 0;          // explicit type, immutable by default
let y = 0;                // inferred type, immutable
let mut z: i32 = 0;       // mutable, explicit type
let mut w = 0;            // mutable, inferred type
const pi: f64 = 3.14159;  // compile-time constant
```

All variables must be initialized. Mutability is opt-in via `let mut`.
Constants are declared with `const` and must be evaluable at compile time.

## Functions

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

## Control Flow

### if / else

```idot
fn classify(x: i32) -> i32 {
    if x < 0 {
        return 111;
    } else {
        return 222;
    }
}
```

`if` / `else` are expressions.

### while loops

```idot
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

`while` is also an expression.

### for..in loops

`for..in` loops iterate over containers.

### match

`match` provides pattern matching with wildcard fallthrough:

```idot
fn describe(x: i32) -> i32 {
    match x {
        0 => { return 10; }
        1 => { return 20; }
        _ => { return 99; }
    }
}
```

## Structs

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

## Examples at a Glance

| File | Shows |
|------|-------|
| `hello.ido` | Minimal program |
| `fib.ido` | Functions, recursion |
| `math.ido` | Arithmetic, function composition |
| `control_flow.ido` | if/else |
| `types.ido` | Variables, type inference, assignment |
| `structs.ido` | Struct types, field access, literals |
| `for_loop.ido` | while loops |
| `generics.ido` | Generic functions |
| `pointers.ido` | Pointer types |
| `defer_example.ido` | Deferred execution |
| `comptime.ido` | Compile-time evaluation |
| `when.ido` | Conditional compilation |
