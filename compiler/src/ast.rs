use std::fmt;
use std::ops::Range;

pub type Span = Range<usize>;

// === RESOLVED TYPES (produced by semantic analysis) ===

#[derive(Debug, Clone, PartialEq)]
pub enum TypeVal {
    Void,
    Bool,
    Int(IntSize),
    Float(FloatSize),
    Ptr(Box<TypeVal>),
    ConstPtr(Box<TypeVal>),
    NullablePtr(Box<TypeVal>),
    ManyPtr(Box<TypeVal>),
    Slice(Box<TypeVal>),
    Array(u64, Box<TypeVal>),
    Optional(Box<TypeVal>),
    ErrorUnion(Box<TypeVal>),
    Fn(Vec<TypeVal>, Option<Box<TypeVal>>),
    Struct(Vec<(String, TypeVal)>),
    Named(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntSize {
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    Isize, Usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatSize {
    F32, F64,
}

// === DECLARATIONS ===

#[derive(Debug, Clone)]
pub enum Decl {
    Let(VarDecl),
    Const(ConstDecl),
    Fn(FnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Union(UnionDecl),
    Import(ImportDecl),
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub mutable: bool,
    pub type_: Option<Type>,
    pub init: Option<Expr>,
    pub resolved_type: Option<TypeVal>,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub type_: Option<Type>,
    pub init: Expr,
    pub resolved_type: Option<TypeVal>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_: Type,
    pub default: Option<Expr>,
    pub resolved_type: Option<TypeVal>,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub is_extern: bool,
    pub foreign_lib: Option<String>,
    pub pub_: bool,
    pub resolved_ret_type: Option<TypeVal>,
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub type_: Type,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub backing_type: Option<Type>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct UnionDecl {
    pub name: String,
    pub tagged: bool,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub path: String,
}

// === TYPES ===

#[derive(Debug, Clone)]
pub enum Type {
    Named(String),
    Ptr(Box<Type>),
    ConstPtr(Box<Type>),
    NullablePtr(Box<Type>),
    ManyPtr(Box<Type>),
    Slice(Box<Type>),
    Array(u64, Box<Type>),
    Optional(Box<Type>),
    ErrorUnion(Box<Type>),
    Fn(Vec<Type>, Option<Box<Type>>),
    Inferred,
}

// === EXPRESSIONS ===

#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(i64),
    FloatLit(f64),
    StrLit(String),
    CharLit(char),
    BoolLit(bool),
    NullLit,
    Ident(String),

    Block(Block),
    If(Box<Expr>, Block, Option<Box<Expr>>),
    For(Box<Expr>, Option<String>, Option<String>, Block),
    While(Box<Expr>, Block),
    Match(Box<Expr>, Vec<MatchArm>, Option<Block>),

    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),

    Call(Box<Expr>, Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Field(Box<Expr>, String),
    Slice(Box<Expr>, Option<Box<Expr>>, Option<Box<Expr>>),

    StructInit(String, Vec<(String, Expr)>),
    ArrayLit(Vec<Expr>),
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub patterns: Vec<MatchPattern>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum MatchPattern {
    Expr(Expr),
    Range(Expr, Expr),
    Wildcard,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

impl Block {
    pub fn empty() -> Self {
        Block { stmts: vec![] }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Decl(Decl),
    Expr(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Neg,
    Not,
    Addr,
    Deref,
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Le => write!(f, "<="),
            BinOp::Ge => write!(f, ">="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::BitAnd => write!(f, "&"),
            BinOp::BitOr => write!(f, "|"),
            BinOp::BitXor => write!(f, "^"),
            BinOp::Shl => write!(f, "<<"),
            BinOp::Shr => write!(f, ">>"),
        }
    }
}
