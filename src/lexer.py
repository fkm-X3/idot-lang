# Tokeniser that builds itself from LangSpec at construction time.
# Adding a keyword, operator, or delimiter to lang.toml is enough —
# no code changes needed here.

from __future__ import annotations
from dataclasses import dataclass
from typing import Iterator
from .spec import LangSpec


# ── Token types ───────────────────────────────────────────────

class TT:  # Token Type namespace
    # literals
    INT    = "INT"
    FLOAT  = "FLOAT"
    STRING = "STRING"
    BOOL   = "BOOL"
    # identifiers & keywords (keywords are identified by value)
    IDENT   = "IDENT"
    KEYWORD = "KEYWORD"
    TYPE    = "TYPE"
    # operators & punctuation (identified by value)
    OP    = "OP"
    DELIM = "DELIM"
    # structure
    EOF = "EOF"


@dataclass
class Token:
    type: str
    value: str
    line: int
    col: int

    def __repr__(self):
        return f"Token({self.type}, {self.value!r}, {self.line}:{self.col})"


# ── Lexer ─────────────────────────────────────────────────────

class Lexer:
    """
    Converts idot source into a flat list of Tokens.
    All language-specific strings (keywords, operators, delimiters,
    type names) come from the LangSpec — nothing is hard-coded.
    """

    def __init__(self, spec: LangSpec):
        self.spec = spec

        # Build operator list sorted longest-first so '>=' is
        # matched before '>'.
        all_op_syms = {op.symbol for op in spec.operators}
        self._operators: list[str] = sorted(all_op_syms, key=lambda s: -len(s))

        # Delimiter characters/strings
        d = spec.delimiters
        self._delimiters: set[str] = {
            d.block_open, d.block_close,
            d.paren_open, d.paren_close,
            d.arg_sep, d.type_sep,
            d.return_type, d.statement_end,
        }
        # Sort delimiters longest-first too (handles '->' before '-')
        self._delim_sorted: list[str] = sorted(self._delimiters, key=lambda s: -len(s))

    # ── Public API ────────────────────────────────────────────

    def tokenize(self, source: str) -> list[Token]:
        return list(self._scan(source))

    # ── Internal scanner ──────────────────────────────────────

    def _scan(self, source: str) -> Iterator[Token]:
        pos = 0
        line = 1
        col = 1
        n = len(source)

        def peek(offset=0) -> str:
            i = pos + offset
            return source[i] if i < n else ""

        def advance(count=1) -> str:
            nonlocal pos, col
            ch = source[pos:pos+count]
            pos += count
            col += count
            return ch

        def newline():
            nonlocal line, col
            line += 1
            col = 1

        while pos < n:
            # ── Whitespace
            if source[pos] in " \t\r":
                advance()
                continue

            if source[pos] == "\n":
                advance()
                newline()
                continue

            # ── Comments  (#  …  end of line)
            if source[pos] == "#":
                while pos < n and source[pos] != "\n":
                    advance()
                continue

            start_line, start_col = line, col

            # ── String literals
            if source[pos] == '"':
                advance()  # consume opening quote
                s = []
                while pos < n and source[pos] != '"':
                    if source[pos] == "\\":
                        advance()
                        esc = advance()
                        s.append({"n": "\n", "t": "\t", "\\": "\\", '"': '"'}.get(esc, esc))
                    else:
                        s.append(advance())
                advance()  # consume closing quote
                yield Token(TT.STRING, "".join(s), start_line, start_col)
                continue

            # ── Numeric literals
            if source[pos].isdigit():
                num = []
                is_float = False
                while pos < n and (source[pos].isdigit() or source[pos] == "."):
                    if source[pos] == ".":
                        is_float = True
                    num.append(advance())
                yield Token(
                    TT.FLOAT if is_float else TT.INT,
                    "".join(num),
                    start_line, start_col,
                )
                continue

            # ── Identifiers, keywords, type names, bool literals
            if source[pos].isalpha() or source[pos] == "_":
                word = []
                while pos < n and (source[pos].isalnum() or source[pos] == "_"):
                    word.append(advance())
                w = "".join(word)

                if w in self.spec.keyword_set:
                    kw = self.spec.keyword_map[w]
                    tt = TT.BOOL if kw.role == "literal" else TT.KEYWORD
                    yield Token(tt, w, start_line, start_col)
                elif w in self.spec.type_set:
                    yield Token(TT.TYPE, w, start_line, start_col)
                else:
                    yield Token(TT.IDENT, w, start_line, start_col)
                continue

            # ── Operators (longest match)
            matched_op = None
            for sym in self._operators:
                if source[pos:pos+len(sym)] == sym:
                    matched_op = sym
                    break

            # ── Delimiters (longest match, checked after operators
            #    so '->' beats '-')
            matched_delim = None
            for sym in self._delim_sorted:
                if source[pos:pos+len(sym)] == sym:
                    matched_delim = sym
                    break

            # Prefer the longer of the two matches
            if matched_op and matched_delim:
                if len(matched_delim) > len(matched_op):
                    matched_op = None
                else:
                    matched_delim = None

            if matched_op:
                advance(len(matched_op))
                yield Token(TT.OP, matched_op, start_line, start_col)
                continue

            if matched_delim:
                advance(len(matched_delim))
                yield Token(TT.DELIM, matched_delim, start_line, start_col)
                continue

            # ── Unknown character
            ch = advance()
            raise SyntaxError(
                f"[idot lexer] Unknown character {ch!r} at {start_line}:{start_col}"
            )

        yield Token(TT.EOF, "", line, col)