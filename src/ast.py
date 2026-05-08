"""AST node definitions for the idot compiler.

These are simple dataclasses used by the parser and the code-generator.
"""

from __future__ import annotations
from dataclasses import dataclass
from typing import List, Optional


class Node:
    pass


@dataclass
class Program(Node):
    functions: List["FuncDef"]


@dataclass
class Param(Node):
    name: str
    type_name: str


@dataclass
class FuncDef(Node):
    name: str
    params: List[Param]
    return_type: str
    body: List[Node]
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class VarDecl(Node):
    name: str
    type_name: str
    value: Optional[Node]
    mutable: bool = False
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class Return(Node):
    value: Optional[Node]
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class If(Node):
    condition: Node
    then_block: List[Node]
    else_block: Optional[List[Node]] = None
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class While(Node):
    condition: Node
    body: List[Node]
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class For(Node):
    var: str
    iterable: Node
    body: List[Node]
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class Assign(Node):
    target: str
    value: Node
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class Call(Node):
    callee: str
    args: List[Node]
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class Identifier(Node):
    name: str
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class IntLiteral(Node):
    value: int
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class FloatLiteral(Node):
    value: float
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class BoolLiteral(Node):
    value: bool
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class StrLiteral(Node):
    value: str
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class BinaryOp(Node):
    op: str
    left: Node
    right: Node
    line: Optional[int] = None
    col: Optional[int] = None


@dataclass
class UnaryOp(Node):
    op: str
    operand: Node
    line: Optional[int] = None
    col: Optional[int] = None
