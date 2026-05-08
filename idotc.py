# idotc — the idot compiler
# Usage:
#     idotc <file.id>              compile -> <file>.c  then run gcc
#     idotc <file.id> --emit-c    only emit C, don't compile
#     idotc <file.id> --tokens    dump lexer output
#     idotc <file.id> --ast       dump AST
#     idotc --spec                dump the loaded language spec


import sys
import argparse
import subprocess
from pathlib import Path

# Allow running from repo root
sys.path.insert(0, str(Path(__file__).parent))

from src import spec as spec_mod
from src.lexer import Lexer
from src.parser import Parser
from src.codegen import CCodegen


def main():
    ap = argparse.ArgumentParser(prog="idotc", description="idot compiler")
    ap.add_argument("file", nargs="?", help="source file (.id)")
    ap.add_argument("--emit-c",   action="store_true", help="emit C only")
    ap.add_argument("--tokens",   action="store_true", help="dump tokens")
    ap.add_argument("--ast",      action="store_true", help="dump AST")
    ap.add_argument("--spec",     action="store_true", help="dump spec info")
    ap.add_argument("--spec-file", default=None, help="path to lang.toml")
    ap.add_argument("-o", "--out", default=None, help="output binary name")
    args = ap.parse_args()

    # Load spec
    spec_path = args.spec_file or (Path(__file__).parent / "lang.toml")
    spec = spec_mod.load(spec_path)

    if args.spec:
        _dump_spec(spec)
        return

    if not args.file:
        ap.print_help()
        sys.exit(1)

    src_path = Path(args.file)
    if not src_path.exists():
        print(f"error: file not found: {src_path}", file=sys.stderr)
        sys.exit(1)

    source = src_path.read_text()

    # Lex
    lexer = Lexer(spec)
    try:
        tokens = lexer.tokenize(source)
    except SyntaxError as e:
        print(f"Lexer error: {e}", file=sys.stderr)
        sys.exit(1)

    if args.tokens:
        for tok in tokens:
            print(tok)
        return

    # Parse
    parser = Parser(tokens, spec)
    try:
        tree = parser.parse()
    except Exception as e:
        print(f"Parse error: {e}", file=sys.stderr)
        sys.exit(1)

    if args.ast:
        _dump_ast(tree)
        return

    # Codegen
    cg = CCodegen(spec)
    c_source = cg.generate(tree)

    c_path = src_path.with_suffix(".c")
    c_path.write_text(c_source, encoding="utf-8")
    print(f"  -> {c_path}")

    if args.emit_c:
        return

    # Compile C → binary
    out_bin = args.out or str(src_path.with_suffix(""))
    result = subprocess.run(
        ["gcc", "-o", out_bin, str(c_path), "-std=c11", "-O2"],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        print("GCC error:", result.stderr, file=sys.stderr)
        sys.exit(1)
    print(f"  -> {out_bin}")


def _dump_spec(spec):
    print(f"=== {spec.name} {spec.version} (target: {spec.target}) ===")
    print(f"\nTypes ({len(spec.types)}):")
    for t in spec.types:
        print(f"  {t.name:12} -> {t.c_type}")
    print(f"\nKeywords ({len(spec.keywords)}):")
    for kw in spec.keywords:
        print(f"  {kw.word:12} [{kw.role}]  {kw.description}")
    print(f"\nOperators ({len(spec.operators)}):")
    for op in spec.operators:
        print(f"  {op.symbol:6} prec={op.prec}  {op.arity:6}  -> C: {op.c_op}")
    print(f"\nBuilt-ins ({len(spec.builtins)}):")
    for b in spec.builtins:
        params = ", ".join(b.params)
        print(f"  {b.name}({params}) -> {b.ret}")


def _dump_ast(node, indent=0):
    prefix = "  " * indent
    name = type(node).__name__
    # Print fields
    fields = {k: v for k, v in vars(node).items() if k not in ("line", "col")}
    children = {}
    simple = {}
    for k, v in fields.items():
        if isinstance(v, list) and v and hasattr(v[0], "__dataclass_fields__"):
            children[k] = v
        elif hasattr(v, "__dataclass_fields__"):
            children[k] = [v]
        else:
            simple[k] = v
    simple_str = "  ".join(f"{k}={v!r}" for k, v in simple.items())
    print(f"{prefix}{name}  {simple_str}")
    for k, nodes in children.items():
        print(f"{prefix}  {k}:")
        for n in nodes:
            _dump_ast(n, indent + 2)


if __name__ == "__main__":
    main()