# Recursive-descent parser driven by the LangSpec.
# Operator precedence table, keyword roles, delimiter characters —
# all sourced from spec, not hard-coded.


from __future__ import annotations
from typing import Optional
from .spec import LangSpec
from .lexer import Token, TT, Lexer
from . import ast as A


class ParseError(Exception):
    pass


class Parser:
    def __init__(self, tokens: list[Token], spec: LangSpec):
        self.tokens = tokens
        self.pos = 0
        self.spec = spec
        d = spec.delimiters
        self._block_open   = d.block_open
        self._block_close  = d.block_close
        self._paren_open   = d.paren_open
        self._paren_close  = d.paren_close
        self._arg_sep      = d.arg_sep
        self._type_sep     = d.type_sep
        self._return_type  = d.return_type
        self._stmt_end     = d.statement_end

    # ── Helpers ───────────────────────────────────────────────

    def peek(self, offset=0) -> Token:
        i = self.pos + offset
        return self.tokens[i] if i < len(self.tokens) else self.tokens[-1]

    def advance(self) -> Token:
        t = self.tokens[self.pos]
        self.pos += 1
        return t

    def check(self, type_: str, value: str | None = None) -> bool:
        t = self.peek()
        if t.type != type_:
            return False
        if value is not None and t.value != value:
            return False
        return True

    def match(self, type_: str, value: str | None = None) -> Optional[Token]:
        if self.check(type_, value):
            return self.advance()
        return None

    def expect(self, type_: str, value: str | None = None) -> Token:
        t = self.match(type_, value)
        if t is None:
            got = self.peek()
            want = f"{type_}" + (f"({value!r})" if value else "")
            raise ParseError(
                f"[idot parser] Expected {want} but got "
                f"{got.type}({got.value!r}) at {got.line}:{got.col}"
            )
        return t

    def skip_stmt_end(self):
        """Consume optional statement terminator."""
        self.match(TT.DELIM, self._stmt_end)

    # ── Top level ─────────────────────────────────────────────

    def parse(self) -> A.Program:
        funcs = []
        while not self.check(TT.EOF):
            funcs.append(self.parse_func())
        return A.Program(functions=funcs)

    # ── Function ──────────────────────────────────────────────

    def parse_func(self) -> A.FuncDef:
        t = self.expect(TT.KEYWORD, "fn")
        name = self.expect(TT.IDENT).value
        self.expect(TT.DELIM, self._paren_open)
        params = self.parse_params()
        self.expect(TT.DELIM, self._paren_close)
        ret_type = "void"
        if self.match(TT.DELIM, self._return_type):
            ret_type = self.expect(TT.TYPE).value
        body = self.parse_block()
        return A.FuncDef(name=name, params=params, return_type=ret_type,
                         body=body, line=t.line, col=t.col)

    def parse_params(self) -> list[A.Param]:
        params = []
        if self.check(TT.DELIM, self._paren_close):
            return params
        params.append(self.parse_param())
        while self.match(TT.DELIM, self._arg_sep):
            params.append(self.parse_param())
        return params

    def parse_param(self) -> A.Param:
        name = self.expect(TT.IDENT).value
        self.expect(TT.DELIM, self._type_sep)
        type_name = self.expect(TT.TYPE).value
        return A.Param(name=name, type_name=type_name)

    # ── Block ─────────────────────────────────────────────────

    def parse_block(self) -> list[A.Node]:
        self.expect(TT.DELIM, self._block_open)
        stmts = []
        while not self.check(TT.DELIM, self._block_close):
            if self.check(TT.EOF):
                raise ParseError("[idot parser] Unexpected EOF inside block")
            stmts.append(self.parse_stmt())
        self.expect(TT.DELIM, self._block_close)
        return stmts

    # ── Statements ────────────────────────────────────────────

    def parse_stmt(self) -> A.Node:
        t = self.peek()

        # variable declaration: let / let mut
        if t.type == TT.KEYWORD and t.value == "let":
            return self.parse_var_decl()

        # return
        if t.type == TT.KEYWORD and t.value == "return":
            return self.parse_return()

        # if
        if t.type == TT.KEYWORD and t.value == "if":
            return self.parse_if()

        # while
        if t.type == TT.KEYWORD and t.value == "while":
            return self.parse_while()

        # for
        if t.type == TT.KEYWORD and t.value == "for":
            return self.parse_for()

        # expression statement (assignment, call, etc.)
        node = self.parse_expr()
        self.skip_stmt_end()
        return node

    def parse_var_decl(self) -> A.VarDecl:
        t = self.expect(TT.KEYWORD, "let")
        mutable = bool(self.match(TT.KEYWORD, "mut"))
        name = self.expect(TT.IDENT).value
        self.expect(TT.DELIM, self._type_sep)
        type_name = self.expect(TT.TYPE).value
        # optional initialiser
        value = None
        if self.match(TT.OP, "="):
            value = self.parse_expr()
        self.skip_stmt_end()
        return A.VarDecl(name=name, type_name=type_name, value=value,
                         mutable=mutable, line=t.line, col=t.col)

    def parse_return(self) -> A.Return:
        t = self.expect(TT.KEYWORD, "return")
        value = None
        if not self.check(TT.DELIM, self._stmt_end) and \
           not self.check(TT.DELIM, self._block_close):
            value = self.parse_expr()
        self.skip_stmt_end()
        return A.Return(value=value, line=t.line, col=t.col)

    def parse_if(self) -> A.If:
        t = self.expect(TT.KEYWORD, "if")
        cond = self.parse_expr()
        then = self.parse_block()
        else_ = None
        if self.match(TT.KEYWORD, "else"):
            else_ = self.parse_block()
        return A.If(condition=cond, then_block=then, else_block=else_,
                    line=t.line, col=t.col)

    def parse_while(self) -> A.While:
        t = self.expect(TT.KEYWORD, "while")
        cond = self.parse_expr()
        body = self.parse_block()
        return A.While(condition=cond, body=body, line=t.line, col=t.col)

    def parse_for(self) -> A.For:
        t = self.expect(TT.KEYWORD, "for")
        var = self.expect(TT.IDENT).value
        self.expect(TT.KEYWORD, "in")
        iterable = self.parse_expr()
        body = self.parse_block()
        return A.For(var=var, iterable=iterable, body=body,
                     line=t.line, col=t.col)

    # ── Expressions (Pratt / precedence climbing) ─────────────

    def parse_expr(self, min_prec: int = 0) -> A.Node:
        left = self.parse_unary()

        while True:
            t = self.peek()
            if t.type != TT.OP:
                break
            op = t.value
            op_prec = self.spec.prec_of(op, "binary")
            if op_prec < 0 or op_prec <= min_prec:
                # Check for assignment separately (right-assoc, very low prec)
                if op == "=" and min_prec == 0:
                    self.advance()
                    right = self.parse_expr(0)
                    if not isinstance(left, A.Identifier):
                        raise ParseError(
                            f"[idot parser] Left side of assignment must be "
                            f"an identifier at {t.line}:{t.col}"
                        )
                    left = A.Assign(target=left.name, value=right,
                                    line=t.line, col=t.col)
                    continue
                break

            assoc = self.spec.assoc_of(op)
            next_min = op_prec if assoc == "right" else op_prec + 1

            # Handle 'and' / 'or' / 'not' as keyword operators
            self.advance()
            right = self.parse_expr(next_min)
            left = A.BinaryOp(op=op, left=left, right=right,
                               line=t.line, col=t.col)

        # Handle keyword operators that appear as KEYWORD tokens
        t = self.peek()
        if t.type == TT.KEYWORD and t.value in ("and", "or"):
            op = t.value
            self.advance()
            right = self.parse_expr(0)
            left = A.BinaryOp(op=op, left=left, right=right,
                               line=t.line, col=t.col)

        return left

    def parse_unary(self) -> A.Node:
        t = self.peek()
        # keyword 'not'
        if t.type == TT.KEYWORD and t.value == "not":
            self.advance()
            operand = self.parse_unary()
            return A.UnaryOp(op="not", operand=operand, line=t.line, col=t.col)
        # operator unary (e.g. negation '-')
        if t.type == TT.OP and t.value in self.spec.unary_ops:
            # Only treat as unary if it's not after an expression.
            # (Simple heuristic: position 0 or after an op/delim)
            self.advance()
            operand = self.parse_primary()
            return A.UnaryOp(op=t.value, operand=operand, line=t.line, col=t.col)
        return self.parse_postfix()

    def parse_postfix(self) -> A.Node:
        """Handle function calls: ident(args...)"""
        node = self.parse_primary()
        if isinstance(node, A.Identifier) and \
                self.check(TT.DELIM, self._paren_open):
            self.advance()  # consume '('
            args = []
            if not self.check(TT.DELIM, self._paren_close):
                args.append(self.parse_expr())
                while self.match(TT.DELIM, self._arg_sep):
                    args.append(self.parse_expr())
            self.expect(TT.DELIM, self._paren_close)
            return A.Call(callee=node.name, args=args,
                          line=node.line, col=node.col)
        return node

    def parse_primary(self) -> A.Node:
        t = self.peek()

        if t.type == TT.INT:
            self.advance()
            return A.IntLiteral(value=int(t.value), line=t.line, col=t.col)

        if t.type == TT.FLOAT:
            self.advance()
            return A.FloatLiteral(value=float(t.value), line=t.line, col=t.col)

        if t.type == TT.BOOL:
            self.advance()
            return A.BoolLiteral(value=(t.value == "true"), line=t.line, col=t.col)

        if t.type == TT.STRING:
            self.advance()
            return A.StrLiteral(value=t.value, line=t.line, col=t.col)

        if t.type == TT.IDENT:
            self.advance()
            return A.Identifier(name=t.value, line=t.line, col=t.col)

        if t.type == TT.DELIM and t.value == self._paren_open:
            self.advance()
            expr = self.parse_expr()
            self.expect(TT.DELIM, self._paren_close)
            return expr

        raise ParseError(
            f"[idot parser] Unexpected token {t.type}({t.value!r}) "
            f"at {t.line}:{t.col}"
        )