# Loads lang.toml and provides structured access to every aspect of the language.
# All other compiler modules import from here — none hard-code any language rules.

import tomllib
from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional


# ── Data classes mirroring lang.toml ─────────────────────────

@dataclass
class TypeSpec:
    name: str
    c_type: str
    category: str  # integer | float | bool | string | void

@dataclass
class KeywordSpec:
    word: str
    role: str        # control | decl | literal | modifier
    description: str

@dataclass
class OperatorSpec:
    symbol: str
    name: str
    prec: int
    assoc: str       # left | right | none
    c_op: str
    arity: str       # binary | unary

@dataclass
class BuiltinSpec:
    name: str
    params: list[str]
    ret: str
    c_impl: str      # $0, $1 … are argument placeholders

@dataclass
class DelimiterSpec:
    block_open: str
    block_close: str
    paren_open: str
    paren_close: str
    arg_sep: str
    type_sep: str
    return_type: str
    statement_end: str

@dataclass
class CodegenSpec:
    include_headers: list[str]
    runtime_guard: str
    entry_point: str
    indent: str

@dataclass
class LangSpec:
    name: str
    version: str
    target: str
    file_ext: str
    types: list[TypeSpec]
    keywords: list[KeywordSpec]
    operators: list[OperatorSpec]
    builtins: list[BuiltinSpec]
    delimiters: DelimiterSpec
    syntax: dict[str, str]
    codegen: CodegenSpec

    # ── Derived helpers (computed once at load time) ──────────

    @property
    def keyword_set(self) -> set[str]:
        return {kw.word for kw in self.keywords}

    @property
    def type_set(self) -> set[str]:
        return {t.name for t in self.types}

    @property
    def type_map(self) -> dict[str, TypeSpec]:
        return {t.name: t for t in self.types}

    @property
    def keyword_map(self) -> dict[str, KeywordSpec]:
        return {kw.word: kw for kw in self.keywords}

    def ops_by_arity(self, arity: str) -> list[OperatorSpec]:
        return [op for op in self.operators if op.arity == arity]

    @property
    def binary_ops(self) -> dict[str, OperatorSpec]:
        """symbol → spec, binary only (longest-first for lexer)"""
        ops = {op.symbol: op for op in self.operators if op.arity == "binary"}
        return dict(sorted(ops.items(), key=lambda x: -len(x[0])))

    @property
    def unary_ops(self) -> dict[str, OperatorSpec]:
        return {op.symbol: op for op in self.operators if op.arity == "unary"}

    @property
    def builtin_map(self) -> dict[str, BuiltinSpec]:
        return {b.name: b for b in self.builtins}

    def prec_of(self, symbol: str, arity: str = "binary") -> int:
        for op in self.operators:
            if op.symbol == symbol and op.arity == arity:
                return op.prec
        return -1

    def assoc_of(self, symbol: str) -> str:
        for op in self.operators:
            if op.symbol == symbol and op.arity == "binary":
                return op.assoc
        return "left"


# ── Loader ────────────────────────────────────────────────────

def load(spec_path: Path | str | None = None) -> LangSpec:
    """
    Load lang.toml from *spec_path* (defaults to repo root lang.toml).
    Returns a fully-populated LangSpec.
    """
    if spec_path is None:
        spec_path = Path(__file__).parent.parent / "lang.toml"
    spec_path = Path(spec_path)

    with open(spec_path, "rb") as f:
        raw = tomllib.load(f)

    meta = raw["meta"]
    delim_raw = raw["delimiters"]
    cg_raw = raw["codegen"]

    return LangSpec(
        name=meta["name"],
        version=meta["version"],
        target=meta["target"],
        file_ext=meta["file_ext"],
        types=[TypeSpec(**t) for t in raw.get("types", [])],
        keywords=[KeywordSpec(**k) for k in raw.get("keywords", [])],
        operators=[OperatorSpec(**o) for o in raw.get("operators", [])],
        builtins=[BuiltinSpec(**b) for b in raw.get("builtins", [])],
        delimiters=DelimiterSpec(
            block_open=delim_raw["block_open"],
            block_close=delim_raw["block_close"],
            paren_open=delim_raw["paren_open"],
            paren_close=delim_raw["paren_close"],
            arg_sep=delim_raw["arg_sep"],
            type_sep=delim_raw["type_sep"],
            return_type=delim_raw["return_type"],
            statement_end=delim_raw["statement_end"],
        ),
        syntax=raw.get("syntax", {}),
        codegen=CodegenSpec(
            include_headers=cg_raw["include_headers"],
            runtime_guard=cg_raw["runtime_guard"],
            entry_point=cg_raw["entry_point"],
            indent=cg_raw["indent"],
        ),
    )